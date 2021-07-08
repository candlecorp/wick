use runtime_utils::*;
use vino_entity::Entity;
use vino_runtime::prelude::*;
use vino_transport::MessageTransport;

#[macro_use]
extern crate tracing;

#[test_env_log::test(actix_rt::test)]
async fn simple_schematic() -> TestResult<()> {
  let (network, _) = init_network_from_yaml("./manifests/simple.yaml").await?;

  let data = hashmap! {
      "input" => "simple string",
  };

  let mut result = network
    .request("simple", Entity::test("simple schematic"), &data)
    .await?;

  println!("Result: {:?}", result);
  let output: MessageTransport = result.remove("output").unwrap();
  println!("Output: {:?}", output);
  equals!(
    output,
    MessageTransport::MessagePack(mp_serialize("simple string")?)
  );
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn echo() -> TestResult<()> {
  let (network, _) = init_network_from_yaml("./manifests/echo.yaml").await?;

  let data = hashmap! {
      "input" => "test-data",
  };

  let mut result = network.request("echo", Entity::test("echo"), &data).await?;

  println!("Result: {:?}", result);
  let output: MessageTransport = result.remove("output").unwrap();
  println!("Output: {:?}", output);
  equals!(
    output,
    MessageTransport::MessagePack(mp_serialize("test-data")?)
  );
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn native_component() -> TestResult<()> {
  let (network, _) = init_network_from_yaml("./manifests/native-component.yaml").await?;

  let data = hashmap! {
      "left" => 42,
      "right" => 302309,
  };

  let mut result = network
    .request("native_component", Entity::test("native component"), &data)
    .await?;

  println!("Result: {:?}", result);
  let output: MessageTransport = result.remove("output").unwrap();
  println!("Output: {:?}", output);
  equals!(
    output,
    MessageTransport::MessagePack(mp_serialize(42 + 302309 + 302309)?)
  );
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn nested_schematics() -> TestResult<()> {
  let (network, _) = init_network_from_yaml("./manifests/nested-schematics.yaml").await?;

  let user_data = "user inputted data";

  let data = hashmap! {
      "parent_input" => user_data,
  };

  let mut result = network
    .request("parent", Entity::test("nested_schematics"), &data)
    .await?;
  println!("Result: {:?}", result);
  let output: MessageTransport = result.remove("parent_output").unwrap();
  println!("Output: {:?}", output);
  equals!(
    output,
    MessageTransport::MessagePack(mp_serialize(user_data)?)
  );
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn wapc_component() -> TestResult<()> {
  let (network, _) = init_network_from_yaml("./manifests/wapc-component.yaml").await?;

  let data = hashmap! {
      "input" => "1234567890",
  };

  let mut result = network
    .request("wapc_component", Entity::test("wapc_component"), &data)
    .await?;

  let output: MessageTransport = result.remove("output").unwrap();
  trace!("output: {:?}", output);
  equals!(
    output,
    MessageTransport::MessagePack(mp_serialize("1234567890")?)
  );

  let data = hashmap! {
      "input" => "1234",
  };
  let mut result = network
    .request("wapc_component", Entity::test("wapc_component"), &data)
    .await?;

  let output: MessageTransport = result.remove("output").unwrap();
  equals!(
    output,
    MessageTransport::Exception("Needs to be longer than 8 characters".to_owned())
  );

  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn short_circuit() -> TestResult<()> {
  let (network, _) = init_network_from_yaml("./manifests/short-circuit.yaml").await?;

  let data = hashmap! {
      "input_port1" => "short",
  };

  let mut result = network
    .request("short_circuit", Entity::test("short circuit"), &data)
    .await?;

  println!("result: {:?}", result);
  let output1: MessageTransport = result.remove("output1").unwrap();
  println!("Output: {:?}", output1);
  equals!(
    output1,
    MessageTransport::Exception("Needs to be longer than 8 characters".to_owned())
  );
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn short_circuit_default() -> TestResult<()> {
  let (network, _) = init_network_from_yaml("./manifests/short-circuit-default.yaml").await?;

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

  println!("result: {:?}", result);
  let output1: MessageTransport = result.remove("output1").unwrap();
  println!("Output: {:?}", output1);
  let result: String = mp_deserialize(&output1.into_bytes()?)?;
  equals!(
    result,
    format!(
      "This is my default. Error was: {}",
      "Needs to be longer than 8 characters"
    )
  );
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn multiple_schematics() -> TestResult<()> {
  let (network, _) = init_network_from_yaml("./manifests/multiple-schematics.yaml").await?;

  let data = hashmap! {
      "left" => 42,
      "right" => 302309,
  };

  let mut result = network
    .request("first_schematic", Entity::test("multi schematics"), &data)
    .await?;

  trace!("result: {:?}", result);
  let output: MessageTransport = result.remove("output").unwrap();
  equals!(
    output,
    MessageTransport::MessagePack(mp_serialize(42 + 302309)?)
  );

  let data = hashmap! {
      "input" => "some string",
  };

  let mut result = network
    .request("second_schematic", Entity::test("multi schematics"), &data)
    .await?;

  println!("Result: {:?}", result);
  let output: MessageTransport = result.remove("output").unwrap();
  println!("Output: {:?}", output);
  equals!(
    output,
    MessageTransport::MessagePack(mp_serialize("some string")?)
  );
  Ok(())
}

#[test_env_log::test(actix_rt::test)]
async fn global_providers() -> TestResult<()> {
  let (network, _) = init_network_from_yaml("./manifests/global-provider-def.yaml").await?;

  let data = hashmap! {
      "input" => "some input",
  };

  let mut result = network
    .request("first_schematic", Entity::test("global providers"), &data)
    .await?;

  trace!("result: {:?}", result);
  let output: MessageTransport = result.remove("output").unwrap();
  equals!(
    output,
    MessageTransport::MessagePack(mp_serialize("some input")?)
  );

  let data = hashmap! {
      "input" => "other input",
  };

  let mut result = network
    .request("second_schematic", Entity::test("global providers"), &data)
    .await?;

  println!("Result: {:?}", result);
  let output: MessageTransport = result.remove("output").unwrap();
  println!("Output: {:?}", output);
  equals!(
    output,
    MessageTransport::MessagePack(mp_serialize("other input")?)
  );
  Ok(())
}
