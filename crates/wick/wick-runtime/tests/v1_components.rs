mod utils;
use std::collections::HashMap;

use serde_json::json;
use utils::*;
use wick_packet::{packet_stream, packets, Packet, RuntimeConfig};

type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn flow_with_inputless_component() -> Result<()> {
  common_test(
    "./tests/manifests/v1/flow_with_inputless_component.yaml",
    packet_stream!(("input", "hello world")),
    "test",
    vec![
      Packet::encode("uuid", "aa38fc21-01bd-ade2-254b-185bf88a15f7"),
      Packet::done("uuid"),
      Packet::encode("output", "hello world"),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn composite_inherit() -> Result<()> {
  common_test(
    "./tests/manifests/v1/composite-inherit.wick",
    packet_stream!(("input", "hello world")),
    "test",
    vec![
      Packet::encode("uuid", "aa38fc21-01bd-ade2-254b-185bf88a15f7"),
      Packet::done("uuid"),
      Packet::encode("output", "hello world"),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_context_passthrough() -> Result<()> {
  test_context_passthrough_base(
    RuntimeConfig::from(HashMap::from([
      ("required".to_owned(), json!("required field")),
      ("optional".to_owned(), json!("optional field")),
    ])),
    RuntimeConfig::from(HashMap::from([
      ("required".to_owned(), json!("required field")),
      ("optional".to_owned(), json!("optional field")),
    ])),
    vec![
      Packet::encode("output", "[from input]root_required: required field, root_optional: optional field, required: required field, optional: optional field"),
      Packet::done("output"),
    ],
  )
  .await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_context_passthrough_root_config_opt() -> Result<()> {
  test_context_passthrough_base(
    RuntimeConfig::from(HashMap::from([("required".to_owned(), json!("required field"))])),
    RuntimeConfig::from(HashMap::from([
      ("required".to_owned(), json!("required field")),
      ("optional".to_owned(), json!("optional field")),
    ])),
    vec![
      Packet::encode("output", "[from input]root_required: required field, root_optional: , required: required field, optional: optional field"),
      Packet::done("output"),
    ],
  )
  .await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_context_passthrough_op_config_opt() -> Result<()> {
  test_context_passthrough_base(
    RuntimeConfig::from(HashMap::from([
      ("required".to_owned(), json!("required field")),
      ("optional".to_owned(), json!("optional field")),
    ])),
    RuntimeConfig::from(HashMap::from([("required".to_owned(), json!("required field"))])),
    vec![
      Packet::encode("output", "[from input]root_required: required field, root_optional: optional field, required: required field, optional: "),
      Packet::done("output"),
    ],
  )
  .await?;

  Ok(())
}

async fn test_context_passthrough_base(
  root_config: RuntimeConfig,
  config: RuntimeConfig,
  expected: Vec<Packet>,
) -> Result<()> {
  test_with_config(
    "./tests/manifests/v1/component-context-vars-passthrough.yaml",
    packets!(("input", "[from input]")).into(),
    "test",
    expected,
    Some(root_config),
    Some(config),
  )
  .await?;

  Ok(())
}
