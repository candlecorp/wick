use std::collections::HashMap;

#[path = "./runtime_utils/mod.rs"]
mod utils;
use utils::*;
use vino_entity::Entity;
use vino_runtime::prelude::TransportWrapper;
use vino_transport::MessageTransport;

#[test_logger::test(actix_rt::test)]
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
  let output: String = msg.payload.try_into()?;

  equals!(output, "simple string");
  Ok(())
}

#[test_logger::test(actix_rt::test)]
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
  let output: String = msg.payload.try_into()?;

  equals!(output, "test-data");
  Ok(())
}

#[test_logger::test(actix_rt::test)]
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

  equals!(output, 42 + 302309 + 302309);
  Ok(())
}

#[test_logger::test(actix_rt::test)]

async fn senders() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/senders.yaml").await?;

  let data: HashMap<String, String> = HashMap::new();

  let mut result = network
    .request("senders", Entity::test("senders"), &data)
    .await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(result.buffered_size(), (0, 0));
  assert_eq!(messages.len(), 1);

  let msg: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.try_into()?;

  equals!(output, "1234512345");
  Ok(())
}

#[test_logger::test(actix_rt::test)]
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
  let output: String = msg.payload.try_into()?;

  println!("uuid: {:?}", output);
  assert_eq!(output.len(), 36);
  Ok(())
}

#[test_logger::test(actix_rt::test)]
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
  let output: String = msg.payload.try_into()?;
  equals!(output, user_data);
  Ok(())
}

#[test_logger::test(actix_rt::test)]
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
  equals!(result, "1234567890");

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

  equals!(
    output.payload,
    MessageTransport::exception("Needs to be longer than 8 characters".to_owned())
  );

  Ok(())
}

#[test_logger::test(actix_rt::test)]
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
    equals!(result, "Hello world");
  }

  Ok(())
}

#[test_logger::test(actix_rt::test)]
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
  equals!(
    output.payload,
    MessageTransport::exception("Needs to be longer than 8 characters".to_owned())
  );
  Ok(())
}

#[test_logger::test(actix_rt::test)]
async fn short_circuit_with_default() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/short-circuit-default.yaml").await?;

  let data = hashmap! {
      "input_port1" => "short",
  };

  let mut result = network
    .request(
      "short_circuit",
      Entity::test("short circuit default"),
      &data,
    )
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
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

#[test_logger::test(actix_rt::test)]
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

  let output: i64 = messages.pop().unwrap().payload.try_into()?;
  equals!(output, 42 + 302309);

  let data = hashmap! {
      "input" => "some string",
  };

  let mut result = network
    .request("second_schematic", Entity::test("multi schematics"), &data)
    .await?;
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;
  println!("Output: {:?}", output);
  equals!(output, "some string");
  Ok(())
}

#[test_logger::test(actix_rt::test)]
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

  equals!(output, "some input");

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
  equals!(output, "other input");
  Ok(())
}

#[test_logger::test(actix_rt::test)]
async fn subnetworks() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/sub-network-parent.yaml").await?;

  let data = hashmap! {
      "input" => "some input",
  };

  let mut result = network
    .request("parent", Entity::test("subnetworks"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;

  equals!(output, "some input");

  Ok(())
}
