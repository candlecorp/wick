mod utils;
use serde_json::json;
use utils::*;
use wick_packet::{packet_stream, Packet};

type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn good_wasm_component_v0() -> Result<()> {
  common_test(
    "./tests/manifests/v0/wasmrs-component.yaml",
    packet_stream!(("input", "1234567890")),
    "test",
    vec![Packet::encode("output", "1234567890"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn good_wasm_component_v1() -> Result<()> {
  test_with_config(
    "../../integration/test-baseline-component/component.yaml",
    packet_stream!(("left", 10), ("right", 1001)),
    "add",
    vec![Packet::encode("output", 1011), Packet::done("output")],
    Some(json!({"default_err":"custom error"}).try_into()?),
    None,
  )
  .await
}

#[test_logger::test(tokio::test)]
#[ignore = "signature check needs to be re-enabled for this test to pass"]
async fn bad_wasm_component_v1() -> Result<()> {
  let path = "./manifests/v1/bad-wasmrs-component.yaml";
  let result = init_engine_from_yaml(path, None).await;
  assert!(result.is_err());
  println!("Error: {:?}", result.err().unwrap());
  Ok(())
}

#[test_logger::test(tokio::test)]
#[ignore = "bad test, need to re-implement in v1 and add better panic handling"]
async fn bad_wasm_component_v0() -> Result<()> {
  common_test(
    "./tests/manifests/v0/bad-wasmrs-component.yaml",
    packet_stream!(("input", "1234567890")),
    "test",
    vec![
      Packet::err(
        "output",
        "Transaction timed out waiting for output from operation error (wick://wasmrs/error)",
      ),
      Packet::done("output"),
    ],
  )
  .await
}
