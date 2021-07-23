pub use maplit::hashmap;
pub use pretty_assertions::assert_eq as equals;
use vino_transport::message_transport::JsonOutput;

pub type TestResult<T> = Result<T, TestError>;

#[macro_use]
extern crate tracing;

use std::collections::HashMap;
use std::fs;

use vino_manifest::{
  Loadable,
  NetworkDefinition,
  NetworkManifest,
  SchematicDefinition,
};
use vino_runtime::network::Network;
use vino_wascap::KeyPair;
pub type TestError = anyhow::Error;
pub use anyhow::*;

pub async fn init_network_from_yaml(path: &str) -> TestResult<(Network, String)> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::from(manifest);
  debug!("Manifest loaded");
  let kp = KeyPair::new_server();

  let network = Network::new(def, &kp.seed()?)?;
  debug!("Initializing network");
  let init = network.init().await;
  info!("Init status : {:?}", init);
  init?;

  let network_id = network.id.clone();
  Ok((network, network_id))
}

pub fn load_network_manifest(path: &str) -> TestResult<NetworkDefinition> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::from(manifest);
  debug!("Manifest loaded");
  Ok(def)
}

pub fn new_schematic(name: &str) -> SchematicDefinition {
  SchematicDefinition {
    name: name.to_owned(),
    ..SchematicDefinition::default()
  }
}

use std::panic;
use std::process::Stdio;

use regex::Regex;
use serde_json::Value;
use tokio::io::{
  AsyncBufReadExt,
  BufReader,
};
use tokio::select;
use tokio::sync::mpsc::{
  self,
  Sender,
};
use tokio::task::JoinHandle;
use tracing::debug;

pub async fn vinoc_invoke(
  port: &str,
  name: &str,
  data: Value,
) -> Result<HashMap<String, JsonOutput>, TestError> {
  debug!("Executing vinoc for schematic {}", name);
  let vinoc_output = tokio_test_bin::get_test_bin("vinoc")
    .env_clear()
    .env("VINO_LOG", "trace")
    .args([
      "invoke",
      name,
      &serde_json::to_string(&data)?,
      "--port",
      port.to_string().as_str(),
      "--trace",
    ])
    .stderr(Stdio::inherit())
    .output()
    .await?;
  debug!(
    "vinoc STDERR is \n {}",
    String::from_utf8_lossy(&vinoc_output.stderr)
  );

  let output = &String::from_utf8_lossy(&vinoc_output.stdout);
  debug!("Result from vinoc is {}", output);

  let result: HashMap<String, JsonOutput> = serde_json::from_str(output)?;

  Ok(result)
}

#[derive(Debug)]
pub enum Signal {
  Kill,
  Continue(String),
}

impl Signal {
  pub fn to_port(self) -> String {
    match self {
      Signal::Continue(s) => s,
      _ => panic!("not a continuation"),
    }
  }
}

pub async fn start_provider(
  name: &str,
  args: &[&str],
  envs: &[(&str, &str)],
) -> Result<(Sender<Signal>, JoinHandle<Result<()>>, String)> {
  debug!("Starting provider bin: {}", name);
  let mut provider = tokio_test_bin::get_test_bin(name)
    .args(args)
    .env_clear()
    .env("RUST_LOG", "debug")
    .envs(envs.to_vec())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

  let stderr = provider.stderr.take().unwrap();
  let stdout = provider.stdout.take().unwrap();

  let mut err_reader = BufReader::new(stderr).lines();
  let mut out_reader = BufReader::new(stdout).lines();

  let (tx, mut rx) = mpsc::channel::<Signal>(1);

  debug!("Spawning task to handle '{}' process", name);
  let name2 = name.to_owned();
  let handle = tokio::spawn(async move {
    select! {
      status = provider.wait() => {
        println!("{} status was: {:?}", name2, status);
        Err(anyhow!("{} stopped early", name2))
      },
      _signal = rx.recv() => {
        println!("{} received signal", name2);
        provider.kill().await?;
        Ok(())
      }
    }
  });

  let re = Regex::new(r"Server bound to 127.0.0.1:(\d+)").unwrap();
  let (tx2, mut rx2) = mpsc::channel::<Signal>(1);

  let name2 = name.to_owned();
  tokio::spawn(async move {
    debug!("Reading '{}' STDOUT", name2);
    while let Ok(Some(l)) = out_reader.next_line().await {
      debug!("{} STDOUT: {}", name2, l);

      if let Some(regex_match) = re.captures(&l) {
        let port = regex_match.get(1).unwrap();
        let _ = tx2.send(Signal::Continue(port.as_str().to_owned())).await;
      }
    }
    debug!("{} Continuing", name2);
  });

  let name2 = name.to_owned();
  tokio::spawn(async move {
    debug!("Reading {} STDERR", name2);
    while let Ok(Some(l)) = err_reader.next_line().await {
      debug!("{} STDERR: {}", name2, l);
    }
    debug!("{} Continuing", name2);
  });

  println!("Waiting for {} to start up", name);
  let port = rx2.recv().await.unwrap();
  println!("{} started, continuing", name);

  Ok((tx, handle, port.to_port()))
}
