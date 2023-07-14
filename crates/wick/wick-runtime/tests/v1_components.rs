mod utils;
use utils::*;
use wick_packet::{packet_stream, Packet};

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
