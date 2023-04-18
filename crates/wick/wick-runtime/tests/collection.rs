use anyhow::Result;
mod utils;
use utils::*;
use wick_packet::{packet_stream, Packet};

#[test_logger::test(tokio::test)]
async fn simple_schematic() -> Result<()> {
  common_test(
    "./manifests/v0/collections/native-component.yaml",
    packet_stream!(("left", 42), ("right", 302309)),
    "native_component",
    vec![Packet::encode("output", 42 + 302309 + 302309), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn global_collections() -> Result<()> {
  common_test(
    "./manifests/v0/global-collection-def.yaml",
    packet_stream!(("input", "some input")),
    "first_schematic",
    vec![Packet::encode("output", "some input"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn subnetworks() -> Result<()> {
  common_test(
    "./manifests/v0/sub-network-parent.yaml",
    packet_stream!(("input", "some input")),
    "parent",
    vec![Packet::encode("output", "some input"), Packet::done("output")],
  )
  .await
}
