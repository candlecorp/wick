use std::env;
use std::sync::Arc;

use runtime_testutils::*;
use vino_entity::Entity;
use vino_invocation_server::{bind_new_socket, make_rpc_server};
use vino_runtime::prelude::TransportWrapper;
type Result<T> = anyhow::Result<T, anyhow::Error>;
use maplit::hashmap;
use pretty_assertions::assert_eq;
#[test_logger::test(tokio::test)]
async fn native_component() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/native-component.yaml").await?;

  let data = hashmap! {
      "left" => 42,
      "right" => 302309,
  };

  let mut result = network
    .request("native_component", Entity::test("native component"), &data)
    .await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(result.buffered_size(), (0, 0));
  assert_eq!(messages.len(), 1);

  let msg: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: i64 = msg.payload.try_into()?;

  assert_eq!(output, 42 + 302309 + 302309);
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn global_providers() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/global-provider-def.yaml").await?;

  let data = hashmap! {
      "input" => "some input",
  };

  let mut result = network
    .request("first_schematic", Entity::test("global providers"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;

  assert_eq!(output, "some input");

  let data = hashmap! {
      "input" => "other input",
  };

  let mut result = network
    .request("second_schematic", Entity::test("global providers"), &data)
    .await?;
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;
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

  let mut result = network.request("parent", Entity::test("subnetworks"), &data).await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;

  assert_eq!(output, "some input");

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn grpc() -> Result<()> {
  let socket = bind_new_socket()?;
  let port = socket.local_addr()?.port();
  let _ = make_rpc_server(socket, Arc::new(test_vino_provider::Provider::default()));
  env::set_var("TEST_PORT", port.to_string());

  let (network, _) = init_network_from_yaml("./manifests/v0/providers/grpc-provider.yaml").await?;
  let user_data = "Hello world";

  let data = hashmap! {
      "input" => user_data,
  };

  let mut result = network.request("grpc", Entity::test("grpc"), &data).await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;

  assert_eq!(output, format!("TEST: {}", user_data));

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn par() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/providers/par.yaml").await?;

  let data = hashmap! {
      "left" => 32,
      "right" => 43,
  };

  let mut result = network.request("par", Entity::test("par"), &data).await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: u32 = messages.pop().unwrap().payload.try_into()?;

  assert_eq!(output, 75);

  Ok(())
}
