mod test;

use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::Result;
use flow_component::Component;
use pretty_assertions::assert_eq;
use serde_json::json;
use test::*;
use wick_packet::{packets, ComponentReference, Entity, Packet, RuntimeConfig};

#[test_logger::test(tokio::test)]
async fn test_forked_input() -> Result<()> {
  let (interpreter, outputs) = test::common_setup(
    "./tests/manifests/v1/behavior-forked-input.yaml",
    "echo",
    packets!(("input", "hello world")),
  )
  .await?;

  let outputs = outputs.into_iter().collect::<Result<Vec<_>, _>>()?;

  assert_eq!(outputs, vec![Packet::done("output")]);

  interpreter.shutdown().await?;

  Ok(())
}
