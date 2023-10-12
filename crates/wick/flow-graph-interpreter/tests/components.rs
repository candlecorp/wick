mod test;

use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::Result;
use flow_component::Component;
use pretty_assertions::assert_eq;
use serde_json::json;
use test::*;
use wick_packet::{packets, ComponentReference, Entity, Packet, PacketExt, RuntimeConfig};

#[test_logger::test(tokio::test)]
async fn test_echo() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/echo.yaml",
    "echo",
    packets!(("input", "hello world")),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap();
  let expected = Packet::encode("output", "hello world");

  assert_eq!(wrapper.unwrap(), expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_no_timeout_downstream() -> Result<()> {
  // this test pipes a packet through an op with an extended timeout and
  // should NOT cause downstream components to time out.
  let sleep_for = 1000;
  let (interpreter, mut outputs) = test::base_setup(
    "./tests/manifests/v1/component-notimeout-downstream.yaml",
    Entity::local("test"),
    packets!(("input", sleep_for)),
    None,
    None,
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap();
  let slept_for: u64 = wrapper?.decode()?;
  assert!(slept_for >= sleep_for);

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_timeout_ok() -> Result<()> {
  let sleep_for = 250;
  let (interpreter, mut outputs) = test::base_setup(
    "./tests/manifests/v1/component-timeout.yaml",
    Entity::local("test"),
    packets!(("input", sleep_for)),
    None,
    None,
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap();
  let slept_for: u64 = wrapper?.decode()?;
  assert!(slept_for >= sleep_for);

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_timeout_fail() -> Result<()> {
  let sleep_for = 1000;
  let (interpreter, mut outputs) = test::base_setup(
    "./tests/manifests/v1/component-timeout.yaml",
    Entity::local("test"),
    packets!(("input", sleep_for)),
    None,
    None,
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap()?;
  let packet = wrapper;
  assert!(packet.unwrap_err().msg().contains("timed out"));

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_context_passing() -> Result<()> {
  let (interpreter, mut outputs) = test::base_setup(
    "./tests/manifests/v1/component-context-vars.yaml",
    Entity::local("test"),
    packets!(("input", "This works!")),
    Some(RuntimeConfig::from(HashMap::from([(
      "component_config_greeting".to_owned(),
      json!("Hello"),
    )]))),
    Some(RuntimeConfig::from(HashMap::from([(
      "op_config_name".to_owned(),
      json!("World"),
    )]))),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap();
  let time = wick_packet::DateTime::from(SystemTime::now());
  use chrono::Datelike;

  let expected = Packet::encode("output", format!("Hello, World! Happy {}! This works!", time.year()));

  assert_eq!(wrapper.unwrap(), expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_anon_nodes() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v1/inline-node-ids.yaml",
    "testop",
    packets!(("input", "Hello world!")),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap();
  let expected = Packet::encode("output", "!DLROW OLLEH");

  assert_eq!(wrapper.unwrap(), expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_sequences() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v1/flow-sequences.yaml",
    "test",
    packets!(("input", "Hello world!")),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap();
  let expected = Packet::encode("output", "!DLROW OLLEH");

  assert_eq!(wrapper.unwrap(), expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_call_component() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v1/component-call.yaml",
    "testop",
    packets!(
      ("message", "Hello world!"),
      (
        "component",
        ComponentReference::new(Entity::test("call_component"), Entity::component("test"))
      )
    ),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap();
  let expected = Packet::encode("output", "!dlrow olleH");

  assert_eq!(wrapper.unwrap(), expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_renamed_ports() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/reverse.yaml",
    "test",
    packets!(("PORT_IN", "hello world")),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap();
  let expected = Packet::encode("PORT_OUT", "dlrow olleh");

  assert_eq!(wrapper.unwrap(), expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_parent_child() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/parent-child.yaml",
    "parent",
    packets!(("parent_input", "hello world")),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap();
  let expected = Packet::encode("parent_output", "DLROW OLLEH");

  assert_eq!(wrapper.unwrap(), expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_parent_child_simple() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/parent-child-simple.yaml",
    "nested_parent",
    packets!(("parent_input", "hello world")),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _wrapper = outputs.pop().unwrap(); //done signal
  let wrapper = outputs.pop().unwrap();
  let expected = Packet::encode("parent_output", "hello world");

  assert_eq!(wrapper.unwrap(), expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_external_collection() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/external.yaml",
    "test",
    packets!(("input", "hello world")),
  )
  .await?;

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let expected = Packet::encode("output", "hello world");

  assert_eq!(wrapper, expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_external_direct() -> Result<()> {
  let (interpreter, mut outputs) = test::base_setup(
    "./tests/manifests/v0/external.yaml",
    Entity::operation("test", "echo"),
    packets!(("input", "hello world")),
    Default::default(),
    Default::default(),
  )
  .await?;

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let expected = Packet::encode("output", "hello world");

  assert_eq!(wrapper, expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_self() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/reference-self.yaml",
    "test",
    packets!(("parent_input", "Hello world")),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let expected = Packet::encode("parent_output", "Hello world");

  assert_eq!(wrapper, expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_spread() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/spread.yaml",
    "test",
    packets!(("input", "Hello world")),
  )
  .await?;

  assert_eq!(outputs.len(), 4);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let expected = Packet::encode("output2", "Hello world");
  assert_eq!(wrapper, expected);
  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let expected = Packet::encode("output1", "Hello world");
  assert_eq!(wrapper, expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_stream() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/stream.yaml",
    "test",
    packets!(("input", "Hello world")),
  )
  .await?;

  assert_eq!(outputs.len(), 6);

  let _ = outputs.pop();
  let expected = Packet::encode("output", "Hello world");

  for wrapper in outputs {
    assert_eq!(wrapper.unwrap(), expected);
  }
  interpreter.shutdown().await?;

  Ok(())
}
#[test_logger::test(tokio::test)]
async fn test_multiple_inputs() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/multiple-inputs.yaml",
    "test",
    packets!(("left", 40), ("right", 10020)),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let expected = Packet::encode("output", 10060);

  assert_eq!(wrapper, expected);

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_stream_multi() -> Result<()> {
  let (interpreter, outputs) = test::common_setup(
    "./tests/manifests/v0/stream-multi.yaml",
    "test",
    packets!(("input", "hello world")),
  )
  .await?;
  assert_eq!(outputs.len(), 13);

  let (mut vowels, mut rest): (Vec<_>, Vec<_>) = outputs
    .into_iter()
    .map(|p| p.unwrap())
    .partition(|wrapper| wrapper.port() == "vowels");
  vowels.pop();
  rest.pop();

  let mut expected_vowels: Vec<_> = "eoo".chars().collect();
  while let Some(ch) = expected_vowels.pop() {
    let wrapper = vowels.pop().unwrap();
    assert_eq!(wrapper, Packet::encode("vowels", ch));
  }

  let mut expected_other: Vec<_> = "hll wrld".chars().collect();
  while let Some(ch) = expected_other.pop() {
    let wrapper = rest.pop().unwrap();
    assert_eq!(wrapper, Packet::encode("rest", ch));
  }
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_subflows() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/children-operations.yaml",
    packets!(("input", "hello WORLD")),
    "DLROW OLLEH",
  )
  .await
}
