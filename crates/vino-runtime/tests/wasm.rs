use runtime_testutils::*;
use tokio_stream::StreamExt;
use vino_entity::Entity;
use vino_runtime::prelude::TransportWrapper;
use vino_transport::MessageTransport;
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
    .request("wapc_component", Entity::test("wapc_component"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: TransportWrapper = messages.pop().unwrap();
  let result: String = output.payload.try_into()?;
  println!("Output for first run: {:?}", result);
  assert_eq!(result, "1234567890");

  let data = hashmap! {
      "input" => "1234",
  };

  println!("Requesting second run");
  let mut result = network
    .request("wapc_component", Entity::test("wapc_component"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: TransportWrapper = messages.pop().unwrap();

  assert_eq!(
    output.payload,
    MessageTransport::exception("Needs to be longer than 8 characters".to_owned())
  );

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
    .request("test", Entity::test("wapc_component"), &data)
    .await?;

  let messages: Vec<TransportWrapper> = result.collect_port("output").await;
  // println!("{:#?}", messages);
  assert_eq!(messages.len(), 5);
  for msg in messages {
    let result: String = msg.payload.try_into()?;
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
    .request("schematic", Entity::test("bad_wapc_component"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect().await;
  println!("{:?}", messages);
  assert_eq!(messages.len(), 1);

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
    .request("ns-link", Entity::test("ns-link"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: TransportWrapper = messages.pop().unwrap();
  let result: String = output.payload.try_into()?;
  println!("Output for first run: {:?}", result);
  assert_eq!(result, "DLROW OLLEH");

  Ok(())
}
