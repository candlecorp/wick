use std::time::{SystemTime, UNIX_EPOCH};

use runtime_testutils::*;
use tokio_stream::StreamExt;
use tracing::debug;
use vino_entity::Entity;
use vino_runtime::prelude::TransportWrapper;
use vino_transport::{Invocation, MessageSignal, MessageTransport};
type Result<T> = anyhow::Result<T, anyhow::Error>;
use maplit::hashmap;
use pretty_assertions::assert_eq;

#[test_logger::test(tokio::test)]
async fn good_wapc_component() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/wapc-component.yaml").await?;

  let data = hashmap! {
      "input" => "1234567890",
  };

  println!("Requesting first run");
  let mut result = network
    .invoke(Invocation::new(
      Entity::test("wapc_component"),
      Entity::schematic("wapc_component"),
      data.try_into()?,
      None,
    ))
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: TransportWrapper = messages.pop().unwrap();
  let result: String = output.payload.deserialize()?;
  println!("Output for first run: {:?}", result);
  assert_eq!(result, "1234567890");

  let data = hashmap! {
      "input" => "1234",
  };

  println!("Requesting second run");
  let mut result = network
    .invoke(Invocation::new(
      Entity::test("wapc_component"),
      Entity::schematic("wapc_component"),
      data.try_into()?,
      None,
    ))
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: TransportWrapper = messages.pop().unwrap();

  assert_eq!(
    output.payload,
    MessageTransport::exception("Needs to be longer than 8 characters")
  );

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn good_wasi_component() -> Result<()> {
  let tempdir = std::env::temp_dir();
  let tempfile = tempdir.join("test_file.txt");

  let now = SystemTime::now();
  let time = now.duration_since(UNIX_EPOCH).unwrap().as_millis().to_string();
  debug!("Writing '{}' to test file {:?}", time, tempfile);
  std::fs::write(&tempfile, &time).unwrap();
  std::env::set_var("TEST_TEMPDIR", tempdir);
  let (network, _) = init_network_from_yaml("./manifests/v0/wasi-component.yaml").await?;
  std::env::remove_var("TEST_TEMPDIR");

  let data = hashmap! {
      "filename" => "/test_file.txt",
  };

  println!("Requesting first run");
  let mut result = network
    .invoke(Invocation::new(
      Entity::test("wapc_component"),
      Entity::schematic("wasi_component"),
      data.try_into()?,
      None,
    ))
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("contents").await;
  assert_eq!(messages.len(), 1);

  let output: TransportWrapper = messages.pop().unwrap();
  let result: String = output.payload.deserialize()?;
  println!("Output for first run: {:?}", result);
  assert_eq!(result, time);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn wapc_stream() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/wapc-stream.yaml").await?;

  let data = hashmap! {
      "input" => "Hello world",
  };

  println!("Requesting first run");
  let mut result = network
    .invoke(Invocation::new(
      Entity::test("wapc_component"),
      Entity::schematic("test"),
      data.try_into()?,
      None,
    ))
    .await?;

  let messages: Vec<TransportWrapper> = result.collect_port("output").await;
  // println!("{:#?}", messages);
  assert_eq!(messages.len(), 5);
  for msg in messages {
    let result: String = msg.payload.deserialize()?;
    assert_eq!(result, "Hello world");
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn bad_wapc_component() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/bad-wapc-component.yaml").await?;

  let data = hashmap! {
      "input" => "1234567890",
  };

  let result = network
    .invoke(Invocation::new(
      Entity::test("bad_wapc_component"),
      Entity::schematic("schematic"),
      data.try_into()?,
      None,
    ))
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect().await;
  println!("{:#?}", messages);
  assert_eq!(messages.len(), 2);

  let output = messages.pop().unwrap();
  assert!(matches!(output.payload, MessageTransport::Signal(MessageSignal::Done)));
  let output: TransportWrapper = messages.pop().unwrap();

  println!("output: {:?}", output);
  assert!(output.payload.is_err());
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn wasm_link_call() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/ns-link-wasm.yaml").await?;

  let data = hashmap! {
      "input" => "hello world",
  };

  println!("Requesting first run");
  let mut result = network
    .invoke(Invocation::new(
      Entity::test("ns-link"),
      Entity::schematic("ns-link"),
      data.try_into()?,
      None,
    ))
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  println!("{:#?}", messages);
  assert_eq!(messages.len(), 1);

  let output = messages.pop().unwrap();
  let result: String = output.payload.deserialize()?;
  println!("Output for first run: {:?}", result);
  assert_eq!(result, "DLROW OLLEH");

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn subnetwork_link_call() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/subnetwork-ns-link.yaml").await?;

  let data = hashmap! {
      "input" => "hello world",
  };

  let mut result = network
    .invoke(Invocation::new(
      Entity::test("ns-link"),
      Entity::schematic("test"),
      data.try_into()?,
      None,
    ))
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  println!("{:#?}", messages);
  assert_eq!(messages.len(), 1);

  let output = messages.pop().unwrap();
  let result: String = output.payload.deserialize()?;
  println!("Output for first run: {:?}", result);
  assert_eq!(result, "DLROW OLLEH");

  Ok(())
}
