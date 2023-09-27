mod utils;
use utils::*;
use wick_packet::{packet_stream, Packet, PacketStream};

type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn simple_schematic() -> Result<()> {
  common_test(
    "./tests/manifests/v0/simple.yaml",
    packet_stream!(("MAIN_IN", "simple string")),
    "simple",
    vec![Packet::encode("MAIN_OUT", "simple string"), Packet::done("MAIN_OUT")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn echo() -> Result<()> {
  common_test(
    "./tests/manifests/v0/echo.yaml",
    packet_stream!(("input", "simple string")),
    "echo",
    vec![Packet::encode("output", "simple string"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn senders() -> Result<()> {
  common_test(
    "./tests/manifests/v0/senders.yaml",
    PacketStream::empty(),
    "senders",
    vec![Packet::encode("output", "1234512345"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn nested_schematics() -> Result<()> {
  common_test(
    "./tests/manifests/v0/nested-schematics.yaml",
    packet_stream!(("parent_input", "simple string")),
    "nested_parent",
    vec![
      Packet::encode("parent_output", "simple string"),
      Packet::done("parent_output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn subnetworks() -> Result<()> {
  common_test(
    "./tests/manifests/v0/sub-network-parent.yaml",
    packet_stream!(("input", "some input")),
    "parent",
    vec![Packet::encode("output", "some input"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn global_collections() -> Result<()> {
  common_test(
    "./tests/manifests/v0/global-collection-def.yaml",
    packet_stream!(("input", "some input")),
    "first_schematic",
    vec![Packet::encode("output", "some input"), Packet::done("output")],
  )
  .await
}
