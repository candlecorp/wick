mod test;

use anyhow::Result;
use flow_component::Component;
use pretty_assertions::assert_eq;
use wick_packet::{packets, Packet};

#[test_logger::test(tokio::test)]
async fn test_forked_input() -> Result<()> {
  let (interpreter, outputs) = test::common_setup(
    "./tests/manifests/v1/behavior-forked-input.yaml",
    "test",
    packets!(("input", "hello world")),
  )
  .await?;

  let outputs = outputs.into_iter().collect::<Result<Vec<_>, _>>()?;

  assert_eq!(outputs, vec![Packet::done("output")]);

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_unused_instances() -> Result<()> {
  let (interpreter, outputs) = test::common_setup(
    "./tests/manifests/v1/behavior-unused-instances.yaml",
    "test",
    packets!(("input", "hello world")),
  )
  .await?;

  let outputs = outputs.into_iter().collect::<Result<Vec<_>, _>>()?;

  assert_eq!(
    outputs,
    vec![Packet::encode("output", "hello world"), Packet::done("output")]
  );

  interpreter.shutdown().await?;

  Ok(())
}
