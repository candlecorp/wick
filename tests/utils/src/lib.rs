pub use maplit::hashmap;
pub use pretty_assertions::assert_eq as equals;
use tokio::time::sleep;
use vino_transport::message_transport::transport_json::TransportJson;

pub type TestResult<T> = Result<T, TestError>;

#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::fs;
use std::time::Duration;

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
  println!("Manifest loaded");
  let kp = KeyPair::new_server();

  let network = Network::new(def, &kp.seed()?)?;
  println!("Initializing network");
  let init = network.init().await;
  info!("Init status : {:?}", init);
  init?;

  let network_id = network.uid.clone();
  Ok((network, network_id))
}

pub fn load_network_manifest(path: &str) -> TestResult<NetworkDefinition> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::from(manifest);
  println!("Manifest loaded");
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

pub async fn vinoc_invoke(
  port: &str,
  name: &str,
  data: Vec<String>,
) -> Result<Vec<HashMap<String, TransportJson>>, TestError> {
  println!("Executing vinoc for schematic {}", name);
  let inputs = data
    .into_iter()
    .flat_map(|kv| vec!["--data".to_owned(), kv])
    .collect::<Vec<String>>();
  println!("Inputs: {:?}", inputs);
  let mut bin = tokio_test_bin::get_test_bin("vinoc");
  let proc = bin
    .env_clear()
    .args([
      "invoke",
      name,
      "--port",
      port.to_string().as_str(),
      "--trace",
    ])
    .args(inputs)
    .stderr(Stdio::inherit());
  println!("Command is {:?}", proc);
  let vinoc_output = proc.output().await?;

  println!(
    "vinoc STDERR is \n {}",
    String::from_utf8_lossy(&vinoc_output.stderr)
  );

  let string = String::from_utf8_lossy(&vinoc_output.stdout);
  println!("Result from vinoc is {:?}", string);
  let output: Vec<_> = string.trim().split('\n').collect();
  println!("Num lines:{:?}", output.len());
  let json: Vec<HashMap<String, TransportJson>> = output
    .iter()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();

  println!("JSON Results: {:?}", json);

  Ok(json)
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
  println!("Starting provider bin: {}", name);

  let mut bin = tokio_test_bin::get_test_bin(name);
  let cmd = bin
    .args(args)
    .env_clear()
    .envs(envs.to_vec())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
  println!("Command is {:?} (ENVS: {:?})", cmd, envs);
  let mut provider = cmd.spawn()?;

  let stderr = provider.stderr.take().unwrap();
  let stdout = provider.stdout.take().unwrap();

  let mut err_reader = BufReader::new(stderr).lines();
  let mut out_reader = BufReader::new(stdout).lines();

  let (tx, mut rx) = mpsc::channel::<Signal>(1);

  let re = Regex::new(r"RPC: Starting server on 127.0.0.1:(\d+)").unwrap();

  let name2 = name.to_owned();
  tokio::spawn(async move {
    println!("Reading '{}' STDOUT", name2);
    while let Ok(Some(l)) = out_reader.next_line().await {
      println!("{} STDOUT: {}", name2, l);
    }
    println!("{} Continuing", name2);
  });

  let (tx2, mut rx2) = mpsc::channel::<Signal>(1);

  let name2 = name.to_owned();
  tokio::spawn(async move {
    println!("Reading {} STDERR", name2);
    while let Ok(Some(l)) = err_reader.next_line().await {
      println!("{} STDERR: {}", name2, l);

      if let Some(regex_match) = re.captures(&l) {
        let port = regex_match.get(1).unwrap();
        let _ = tx2.send(Signal::Continue(port.as_str().to_owned())).await;
      }
    }
    println!("{} Continuing", name2);
  });

  println!("Spawning task to handle '{}' process", name);
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

  println!("Waiting for {} to start up", name);
  sleep(Duration::from_millis(100)).await;

  let port = rx2.recv().await.unwrap();
  println!("{} started, continuing", name);

  Ok((tx, handle, port.to_port()))
}
