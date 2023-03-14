use runtime_testutils::*;
use wick_packet::{packet_stream, Packet, PacketStream};

type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn simple_schematic() -> Result<()> {
  tester(
    "./manifests/v0/simple.yaml",
    packet_stream!(("MAIN_IN", "simple string")),
    "simple",
    vec![Packet::encode("MAIN_OUT", "simple string"), Packet::done("MAIN_OUT")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn echo() -> Result<()> {
  tester(
    "./manifests/v0/echo.yaml",
    packet_stream!(("input", "simple string")),
    "echo",
    vec![Packet::encode("output", "simple string"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn senders() -> Result<()> {
  tester(
    "./manifests/v0/senders.yaml",
    PacketStream::default(),
    "senders",
    vec![Packet::encode("output", "1234512345"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn no_inputs() -> Result<()> {
  tester(
    "./manifests/v0/no-inputs.yaml",
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
async fn nested_schematics() -> Result<()> {
  tester(
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
  tester(
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
#[ignore = "TODO:FIX_HUNG"]
async fn short_circuit_with_default() -> Result<()> {
  tester(
    "./manifests/v0/short-circuit-default.yaml",
    packet_stream!(("input_port1", "short")),
    "short_circuit",
    vec![Packet::err("output", "udnno")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn multiple_inputs() -> Result<()> {
  tester(
    "./manifests/v0/multiple-inputs.yaml",
    packet_stream!(("left", 42), ("right", 302309)),
    "test",
    vec![Packet::encode("output", 42 + 302309), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn subnetworks() -> Result<()> {
  tester(
    "./manifests/v0/sub-network-parent.yaml",
    packet_stream!(("input", "some input")),
    "parent",
    vec![Packet::encode("output", "some input"), Packet::done("output")],
  )
  .await
}
