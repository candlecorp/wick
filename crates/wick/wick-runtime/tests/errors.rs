use anyhow::Result;
mod utils;
use utils::*;
use wick_packet::{packet_stream, Packet};

#[test_logger::test(tokio::test)]
async fn panics() -> Result<()> {
  common_test(
    "./tests/manifests/v0/errors/panics.yaml",
    packet_stream!(("input", "this is my message")),
    "panics",
    vec![
      Packet::err(
        "output",
        "operation produced no output, likely due to a panic or misconfiguration",
      ),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn errors() -> Result<()> {
  common_test(
    "./tests/manifests/v0/errors/errors.yaml",
    packet_stream!(("input", "input")),
    "errors",
    vec![
      Packet::err("output", "Needs to be longer than 8 characters"),
      Packet::done("output"),
    ],
  )
  .await
}
