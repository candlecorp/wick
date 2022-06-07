use std::env;
use std::sync::Arc;

use runtime_testutils::*;
use wasmflow_entity::Entity;
use wasmflow_invocation_server::{bind_new_socket, make_rpc_server};
use wasmflow_runtime::prelude::TransportWrapper;
type Result<T> = anyhow::Result<T, anyhow::Error>;
use maplit::hashmap;
use pretty_assertions::assert_eq;
use wasmflow_invocation::Invocation;
#[test_logger::test(tokio::test)]
async fn native_component() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/collections/native-component.yaml").await?;

  let data = hashmap! {
      "left" => 42,
      "right" => 302309,
  };

  let mut result = network
    .invoke(Invocation::new(
      Entity::test("native component"),
      Entity::local("native_component"),
      data.try_into()?,
      None,
    ))
    .await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<TransportWrapper> = result.drain_port("output").await?;
  assert_eq!(result.buffered_size(), (0, 0));
  assert_eq!(messages.len(), 1);

  let msg: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: i64 = msg.payload.deserialize()?;

  assert_eq!(output, 42 + 302309 + 302309);
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn global_collections() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/global-collection-def.yaml").await?;

  let data = hashmap! {
      "input" => "some input",
  };

  let mut result = network
    .invoke(Invocation::new(
      Entity::test("global collections"),
      Entity::local("first_schematic"),
      data.try_into()?,
      None,
    ))
    .await?;

  let mut messages: Vec<TransportWrapper> = result.drain_port("output").await?;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.deserialize()?;

  assert_eq!(output, "some input");

  let data = hashmap! {
      "input" => "other input",
  };

  let mut result = network
    .invoke(Invocation::new(
      Entity::test("global collections"),
      Entity::local("second_schematic"),
      data.try_into()?,
      None,
    ))
    .await?;
  let mut messages: Vec<TransportWrapper> = result.drain_port("output").await?;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.deserialize()?;
  println!("Output: {:?}", output);
  assert_eq!(output, "other input");
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn subnetworks() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/sub-network-parent.yaml").await?;

  let data = hashmap! {
      "input" => "some input",
  };

  let mut result = network
    .invoke(Invocation::new(
      Entity::test("subnetworks"),
      Entity::local("parent"),
      data.try_into()?,
      None,
    ))
    .await?;

  let mut messages: Vec<TransportWrapper> = result.drain_port("output").await?;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.deserialize()?;

  assert_eq!(output, "some input");

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn grpc() -> Result<()> {
  let socket = bind_new_socket()?;
  let port = socket.local_addr()?.port();
  let _ = make_rpc_server(socket, Arc::new(test_native_collection::Collection::default()));
  env::set_var("TEST_PORT", port.to_string());

  let (network, _) = init_network_from_yaml("./manifests/v0/collections/grpc.yaml").await?;
  let user_data = "Hello world";

  let data = hashmap! {
      "input" => user_data,
  };

  let mut result = network
    .invoke(Invocation::new(
      Entity::test("grpc"),
      Entity::local("grpc"),
      data.try_into()?,
      None,
    ))
    .await?;

  let mut messages: Vec<TransportWrapper> = result.drain_port("output").await?;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.deserialize()?;

  assert_eq!(output, format!("TEST: {}", user_data));

  Ok(())
}

#[test_logger::test(tokio::test)]
#[ignore] // Need to automate the creation of par bundles
async fn par() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/collections/par.yaml").await?;

  let data = hashmap! {
      "left" => 32,
      "right" => 43,
  };

  let mut result = network
    .invoke(Invocation::new(
      Entity::test("par"),
      Entity::local("par"),
      data.try_into()?,
      None,
    ))
    .await?;

  let mut messages: Vec<TransportWrapper> = result.drain_port("output").await?;
  assert_eq!(messages.len(), 1);

  let output: u32 = messages.pop().unwrap().payload.deserialize()?;

  assert_eq!(output, 75);

  Ok(())
}
