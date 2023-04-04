use anyhow::Result;
mod utils;
use utils::*;
use wick_packet::{packet_stream, Packet};

#[test_logger::test(tokio::test)]
async fn panics() -> Result<()> {
  common_test(
    "./manifests/v0/errors/panics.yaml",
    packet_stream!(("input", "input")),
    "panics",
    vec![
      Packet::err("output", "component failed to produce output"),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn errors() -> Result<()> {
  common_test(
    "./manifests/v0/errors/errors.yaml",
    packet_stream!(("input", "input")),
    "errors",
    vec![
      Packet::err("output", "This component will always error"),
      Packet::done("output"),
    ],
  )
  .await
}
