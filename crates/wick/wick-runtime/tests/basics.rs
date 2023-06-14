mod utils;
use utils::*;
use wick_packet::{packet_stream, Packet, PacketStream};

type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn simple_schematic() -> Result<()> {
  common_test(
    "./manifests/v0/simple.yaml",
    packet_stream!(("MAIN_IN", "simple string")),
    "simple",
    vec![Packet::encode("MAIN_OUT", "simple string"), Packet::done("MAIN_OUT")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn echo() -> Result<()> {
  common_test(
    "./manifests/v0/echo.yaml",
    packet_stream!(("input", "simple string")),
    "echo",
    vec![Packet::encode("output", "simple string"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn senders() -> Result<()> {
  common_test(
    "./manifests/v0/senders.yaml",
    PacketStream::empty(),
    "senders",
    vec![Packet::encode("output", "1234512345"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn nested_schematics() -> Result<()> {
  common_test(
    "./manifests/v0/nested-schematics.yaml",
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
async fn short_circuit_to_output() -> Result<()> {
  common_test(
    "./manifests/v0/short-circuit.yaml",
    packet_stream!(("input", "short")),
    "short_circuit",
    vec![
      Packet::err("output", "Needs to be longer than 8 characters"),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn multiple_inputs() -> Result<()> {
  common_test(
    "./manifests/v0/multiple-inputs.yaml",
    packet_stream!(("left", 42), ("right", 302309)),
    "test",
    vec![Packet::encode("output", 42 + 302309), Packet::done("output")],
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
