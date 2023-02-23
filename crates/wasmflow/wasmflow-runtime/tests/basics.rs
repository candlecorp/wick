use runtime_testutils::*;
use wasmflow_packet_stream::{packet_stream, Packet, PacketStream};

type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn simple_schematic() -> Result<()> {
  tester(
    "./manifests/v0/simple.wafl",
    packet_stream!(("input", "simple string")),
    "simple",
    vec![Packet::encode("output", "simple string"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn echo() -> Result<()> {
  tester(
    "./manifests/v0/echo.wafl",
    packet_stream!(("input", "simple string")),
    "echo",
    vec![Packet::encode("output", "simple string"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn senders() -> Result<()> {
  tester(
    "./manifests/v0/senders.wafl",
    PacketStream::default(),
    "senders",
    vec![Packet::encode("output", "1234512345"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn no_inputs() -> Result<()> {
  tester(
    "./manifests/v0/no-inputs.wafl",
    PacketStream::default(),
    "uuid",
    vec![
      Packet::encode("output", "611830d3-641a-68f9-4a69-0dcc25d1f4b0"),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
#[ignore = "TODO:IMPORTANT"]
async fn nested_schematics() -> Result<()> {
  tester(
    "./manifests/v0/nested-schematics.wafl",
    packet_stream!(("parent_input", "simple string")),
    "nested_parent",
    vec![Packet::encode("parent_output", "simple string"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
#[ignore = "TODO:FIX_HUNG"]
async fn short_circuit_to_output() -> Result<()> {
  tester(
    "./manifests/v0/short-circuit.wafl",
    packet_stream!(("input", "short")),
    "short_circuit",
    vec![Packet::err("output", "udnno")],
  )
  .await
}

#[test_logger::test(tokio::test)]
#[ignore = "TODO:FIX_HUNG"]
async fn short_circuit_with_default() -> Result<()> {
  tester(
    "./manifests/v0/short-circuit-default.wafl",
    packet_stream!(("input_port1", "short")),
    "short_circuit",
    vec![Packet::err("output", "udnno")],
  )
  .await
}

#[test_logger::test(tokio::test)]
#[ignore = "TODO:FIX_SEND_AFTER_DONE"]
async fn multiple_schematics() -> Result<()> {
  tester(
    "./manifests/v0/multiple-schematics.wafl",
    packet_stream!(("left", 42), ("right", 302309)),
    "first_schematic",
    vec![Packet::encode("output", 42 + 302309), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn subnetworks() -> Result<()> {
  tester(
    "./manifests/v0/sub-network-parent.wafl",
    packet_stream!(("input", "some input")),
    "parent",
    vec![Packet::encode("output", "some input"), Packet::done("output")],
  )
  .await
}
