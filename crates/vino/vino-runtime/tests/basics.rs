use std::collections::HashMap;

use runtime_testutils::*;
use tokio_stream::StreamExt;
use vino_entity::Entity;
use vino_runtime::prelude::TransportWrapper;
use vino_transport::MessageTransport;

type Result<T> = anyhow::Result<T, anyhow::Error>;
use maplit::hashmap;
use pretty_assertions::assert_eq;

#[test_logger::test(tokio::test)]
async fn simple_schematic() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/simple.yaml").await?;

  let data = hashmap! {
      "input" => "simple string",
  };

  let mut result = network
    .request("simple", Entity::test("simple schematic"), &data)
    .await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(result.buffered_size(), (0, 0));
  assert_eq!(messages.len(), 1);

  let msg: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.deserialize()?;

  assert_eq!(output, "simple string");
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn echo() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/echo.yaml").await?;

  let data = hashmap! {
      "input" => "test-data",
  };

  let mut result = network.request("echo", Entity::test("echo"), &data).await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(result.buffered_size(), (0, 0));
  assert_eq!(messages.len(), 1);

  let msg: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.deserialize()?;

  assert_eq!(output, "test-data");
  Ok(())
}

#[test_logger::test(tokio::test)]

async fn senders() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/senders.yaml").await?;

  let data: HashMap<String, String> = HashMap::new();

  let mut result = network.request("senders", Entity::test("senders"), &data).await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(result.buffered_size(), (0, 0));
  assert_eq!(messages.len(), 1);

  let msg: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.deserialize()?;

  assert_eq!(output, "1234512345");
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn no_inputs() -> Result<()> {
  println!("Running no_inputs test");
  let (network, _) = init_network_from_yaml("./manifests/v0/no-inputs.yaml").await?;

  let data: HashMap<String, String> = HashMap::new();

  let mut result = network.request("uuid", Entity::test("test"), &data).await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(result.buffered_size(), (0, 0));
  assert_eq!(messages.len(), 1);

  let msg: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.deserialize()?;

  println!("uuid: {:?}", output);
  assert_eq!(output.len(), 36);
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn nested_schematics() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/nested-schematics.yaml").await?;

  let user_data = "user inputted data";

  let data = hashmap! {
      "parent_input" => user_data,
  };

  let mut result = network
    .request("nested_parent", Entity::test("nested_schematics"), &data)
    .await?;
  println!("Result: {:?}", result);
  let mut messages: Vec<TransportWrapper> = result.collect_port("parent_output").await;
  assert_eq!(result.buffered_size(), (0, 0));
  assert_eq!(messages.len(), 1);

  let msg: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.deserialize()?;
  assert_eq!(output, user_data);
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn short_circuit_to_output() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/short-circuit.yaml").await?;

  let data = hashmap! {
      "input" => "short",
  };

  let mut result = network
    .request("short_circuit", Entity::test("short circuit"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", output);
  assert_eq!(
    output.payload,
    MessageTransport::exception("Needs to be longer than 8 characters".to_owned())
  );
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn short_circuit_with_default() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/short-circuit-default.yaml").await?;

  let data = hashmap! {
      "input_port1" => "short",
  };

  let mut result = network
    .request("short_circuit", Entity::test("short circuit default"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.deserialize()?;
  println!("Output: {:?}", output);
  assert_eq!(
    output,
    format!(
      "This is my default. Error was: {}",
      "Needs to be longer than 8 characters"
    )
  );
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn multiple_schematics() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/multiple-schematics.yaml").await?;

  let data = hashmap! {
      "left" => 42,
      "right" => 302309,
  };

  let mut result = network
    .request("first_schematic", Entity::test("multi schematics"), &data)
    .await?;
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: i64 = messages.pop().unwrap().payload.deserialize()?;
  assert_eq!(output, 42 + 302309);

  let data = hashmap! {
      "input" => "some string",
  };

  let mut result = network
    .request("second_schematic", Entity::test("multi schematics"), &data)
    .await?;
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.deserialize()?;
  println!("Output: {:?}", output);
  assert_eq!(output, "some string");
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn subnetworks() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/sub-network-parent.yaml").await?;

  let data = hashmap! {
      "input" => "some input",
  };

  let result = network.request("parent", Entity::test("subnetworks"), &data).await?;

  let messages: Vec<_> = result.collect().await;

  println!("{:?}", messages);

  assert_eq!(messages.len(), 2);

  let output: String = messages[0].payload.clone().deserialize()?;

  assert_eq!(output, "some input");

  Ok(())
}
