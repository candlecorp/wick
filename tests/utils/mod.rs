#![allow(unused)]
use std::time::Duration;

use tokio::time::sleep;

pub type TestError = anyhow::Error;
use std::panic;
use std::process::Stdio;

pub use anyhow::{anyhow, Result};
use regex::Regex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::select;
use tokio::sync::mpsc::{self, Sender};
use tokio::task::JoinHandle;

macro_rules! test_cases {
  (tests: $kind:literal, []) => {};
  (tests: $kind:literal, [$($file:literal),*]) => {
    static DIR: &str = env!("CARGO_CRATE_NAME");

    #[rstest::rstest]
    $(#[case($file)])*
    fn test(#[case] file: &'static str) {
      let kind = $kind;
      let file = format!("tests/{}/{}/{}", DIR, kind, file);

      trycmd::TestCases::new().case(file);
    }
  };
  (unit: [$($file:tt),*], integration: [$($int_file:tt),*]) => {

    crate::utils::test_cases!(tests: "unit", [$($file),*]);

    mod integration_test {
      crate::utils::test_cases!(tests: "integration", [$($int_file),*]);
    }

  };
}

pub(crate) use test_cases;

pub async fn wick_invoke(port: &str, name: &str, data: Vec<String>) -> Result<Vec<serde_json::Value>, TestError> {
  println!("Executing wick for schematic {}", name);
  let inputs = data
    .into_iter()
    .flat_map(|kv| vec!["--data".to_owned(), kv])
    .collect::<Vec<String>>();
  println!("Inputs: {:?}", inputs);

  let mut bin = tokio::process::Command::from(test_bin::get_test_bin("wick"));

  let proc = bin
    .env_clear()
    .args(["rpc", "invoke", name, "--port", port.to_string().as_str(), "--trace"])
    .args(inputs)
    .stderr(Stdio::inherit());
  println!("Command is {:?}", proc);
  let wick_output = proc.output().await?;

  println!("wick STDERR is \n {}", String::from_utf8_lossy(&wick_output.stderr));

  let string = String::from_utf8_lossy(&wick_output.stdout);
  println!("Result from wick is {:?}", string);
  let output: Vec<_> = string.trim().split('\n').collect();
  println!("Num lines:{:?}", output.len());
  let json: Vec<serde_json::Value> = output.iter().map(|l| serde_json::from_str(l).unwrap()).collect();

  println!("JSON Results: {:?}", json);

  Ok(json)
}

#[derive(Debug)]
pub enum Signal {
  Kill,
  Continue(String),
}

impl Signal {
  pub fn into_port(self) -> String {
    match self {
      Signal::Continue(s) => s,
      _ => panic!("not a continuation"),
    }
  }
}

pub async fn start_collection(
  binary: &str,
  name: &str,
  args: &[&str],
  envs: &[(&str, &str)],
) -> Result<(Sender<Signal>, JoinHandle<Result<()>>, String)> {
  println!("Starting collection bin: {} ({})", binary, name);

  let mut bin = tokio::process::Command::from(test_bin::get_test_bin(binary));
  let cmd = bin
    .args(args)
    .env_clear()
    .envs(envs.to_vec())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
  println!("Command is {:?} (ENVS: {:?})", cmd, envs);
  let mut collection = cmd.spawn()?;

  let stderr = collection.stderr.take().unwrap();
  let stdout = collection.stdout.take().unwrap();

  let mut err_reader = BufReader::new(stderr).lines();
  let mut out_reader = BufReader::new(stdout).lines();

  let (tx, mut rx) = mpsc::channel::<Signal>(1);

  let re = Regex::new(r"GRPC server bound to 127.0.0.1 on port (\d+)").unwrap();

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
      status = collection.wait() => {
        println!("{} status was: {:?}", name2, status);
        Err(anyhow!("{} stopped early", name2))
      },
      _signal = rx.recv() => {
        println!("{} received signal", name2);
        collection.kill().await?;
        Ok(())
      }
    }
  });

  println!("Waiting for {} to start up", name);
  sleep(Duration::from_millis(100)).await;

  let port = rx2.recv().await.unwrap();
  println!("{} started, continuing", name);

  Ok((tx, handle, port.into_port()))
}
