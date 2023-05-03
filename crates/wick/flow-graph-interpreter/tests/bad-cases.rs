#![allow(unused_attributes, clippy::box_default)]

mod test;
use anyhow::Result;
use pretty_assertions::assert_eq;
use wick_packet::{packets, Packet};

async fn test_failure(file: &str, errstr: &str) -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(file, "test", packets!(("input", "Hello world"))).await?;

  assert_eq!(outputs.len(), 2);

  outputs.pop();
  let p = outputs.pop().unwrap().unwrap();
  assert_eq!(p, Packet::err("output", errstr));

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_panic() -> Result<()> {
  test_failure(
    "./tests/manifests/v0/bad-cases/panic.yaml",
    "Operation wick://test/panic panicked",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_error() -> Result<()> {
  test_failure(
    "./tests/manifests/v0/bad-cases/error.yaml",
    "Operation wick://test/error failed: Component error: This operation always errors",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_timeout_done_noclose() -> Result<()> {
  test_failure(
    "./tests/manifests/v0/bad-cases/timeout-done-noclose.yaml",
    "component failed to produce output",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_noimpl() -> Result<()> {
  test_failure(
    "./tests/manifests/v0/bad-cases/noimpl.yaml",
    "component failed to produce output",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_component_error() -> Result<()> {
  test_failure("./tests/manifests/v0/bad-cases/comp-error.yaml", "Oh no").await
}
