use std::collections::HashMap;

#[path = "./runtime_utils/mod.rs"]
mod utils;
use tokio_stream::StreamExt;
use utils::*;
use vino_entity::Entity;
use vino_runtime::prelude::InvocationTransport;
use vino_transport::MessageTransport;

#[test_env_log::test(actix_rt::test)]
async fn simple_schematic() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/network/simple.yaml").await?;

  let data = hashmap! {
      "input" => "simple string",
  };

  let result = network
    .request("simple", Entity::test("simple schematic"), &data)
    .await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let msg: InvocationTransport = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.try_into()?;

  equals!(output, "simple string");
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn echo() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/network/echo.yaml").await?;

  let data = hashmap! {
      "input" => "test-data",
  };

  let result = network.request("echo", Entity::test("echo"), &data).await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let msg: InvocationTransport = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.try_into()?;

  equals!(output, "test-data");
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn native_component() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/network/native-component.yaml").await?;

  let data = hashmap! {
      "left" => 42,
      "right" => 302309,
  };

  let result = network
    .request("native_component", Entity::test("native component"), &data)
    .await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let msg: InvocationTransport = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: i64 = msg.payload.try_into()?;

  equals!(output, 42 + 302309 + 302309);
  Ok(())
}

#[test_env_log::test(actix_rt::test)]

async fn defaults() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/network/defaults.yaml").await?;

  let data: HashMap<String, String> = HashMap::new();

  let result = network
    .request("defaults", Entity::test("defaults"), &data)
    .await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let msg: InvocationTransport = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: i64 = msg.payload.try_into()?;

  equals!(output, 1234512345);
  Ok(())
}

#[test_env_log::test(actix_rt::test)]

async fn no_inputs() -> Result<()> {
  println!("Running no_inputs test");
  let (network, _) = init_network_from_yaml("./manifests/v0/network/no-inputs.yaml").await?;

  let data: HashMap<String, String> = HashMap::new();

  let result = network.request("test", Entity::test("test"), &data).await?;
  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let msg: InvocationTransport = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.try_into()?;

  println!("uuid: {:?}", output);
  assert_eq!(output.len(), 36);
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn nested_schematics() -> Result<()> {
  let (network, _) =
    init_network_from_yaml("./manifests/v0/network/nested-schematics.yaml").await?;

  let user_data = "user inputted data";

  let data = hashmap! {
      "parent_input" => user_data,
  };

  let result = network
    .request("parent", Entity::test("nested_schematics"), &data)
    .await?;
  println!("Result: {:?}", result);
  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let msg: InvocationTransport = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.try_into()?;
  equals!(output, user_data);
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn good_wapc_component() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/network/wapc-component.yaml").await?;

  let data = hashmap! {
      "input" => "1234567890",
  };

  println!("Requesting first run");
  let result = network
    .request("wapc_component", Entity::test("wapc_component"), &data)
    .await?;

  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let output: InvocationTransport = messages.pop().unwrap();
  let result: String = output.payload.try_into()?;
  println!("Output for first run: {:?}", result);
  equals!(result, "1234567890");

  let data = hashmap! {
      "input" => "1234",
  };

  println!("Requesting second run");
  let result = network
    .request("wapc_component", Entity::test("wapc_component"), &data)
    .await?;

  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let output: InvocationTransport = messages.pop().unwrap();

  equals!(
    output.payload,
    MessageTransport::Exception("Needs to be longer than 8 characters".to_owned())
  );

  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn bad_wapc_component() -> Result<()> {
  let (network, _) =
    init_network_from_yaml("./manifests/v0/network/bad-wapc-component.yaml").await?;

  let data = hashmap! {
      "input" => "1234567890",
  };

  let result = network
    .request("schematic", Entity::test("bad_wapc_component"), &data)
    .await?;

  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let output: InvocationTransport = messages.pop().unwrap();

  println!("output: {:?}", output);
  assert!(output.payload.is_err());
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn short_circuit_to_output() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/network/short-circuit.yaml").await?;

  let data = hashmap! {
      "input" => "short",
  };

  let result = network
    .request("short_circuit", Entity::test("short circuit"), &data)
    .await?;

  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let output: InvocationTransport = messages.pop().unwrap();
  println!("Output: {:?}", output);
  equals!(
    output.payload,
    MessageTransport::Exception("Needs to be longer than 8 characters".to_owned())
  );
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn short_circuit_with_default() -> Result<()> {
  let (network, _) =
    init_network_from_yaml("./manifests/v0/network/short-circuit-default.yaml").await?;

  let data = hashmap! {
      "input_port1" => "short",
  };

  let result = network
    .request(
      "short_circuit",
      Entity::test("short circuit default"),
      &data,
    )
    .await?;

  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;
  println!("Output: {:?}", output);
  equals!(
    output,
    format!(
      "This is my default. Error was: {}",
      "Needs to be longer than 8 characters"
    )
  );
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn multiple_schematics() -> Result<()> {
  let (network, _) =
    init_network_from_yaml("./manifests/v0/network/multiple-schematics.yaml").await?;

  let data = hashmap! {
      "left" => 42,
      "right" => 302309,
  };

  let result = network
    .request("first_schematic", Entity::test("multi schematics"), &data)
    .await?;
  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let output: i64 = messages.pop().unwrap().payload.try_into()?;
  equals!(output, 42 + 302309);

  let data = hashmap! {
      "input" => "some string",
  };

  let result = network
    .request("second_schematic", Entity::test("multi schematics"), &data)
    .await?;
  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;
  println!("Output: {:?}", output);
  equals!(output, "some string");
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn global_providers() -> Result<()> {
  let (network, _) =
    init_network_from_yaml("./manifests/v0/network/global-provider-def.yaml").await?;

  let data = hashmap! {
      "input" => "some input",
  };

  let result = network
    .request("first_schematic", Entity::test("global providers"), &data)
    .await?;

  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;

  equals!(output, "some input");

  let data = hashmap! {
      "input" => "other input",
  };

  let result = network
    .request("second_schematic", Entity::test("global providers"), &data)
    .await?;
  let mut messages: Vec<InvocationTransport> = result.collect().await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;
  println!("Output: {:?}", output);
  equals!(output, "other input");
  Ok(())
}
