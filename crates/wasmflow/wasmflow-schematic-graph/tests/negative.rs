mod test;
use anyhow::Result;
use test::*;

#[test_logger::test(tokio::test)]
async fn test_multiple_inputs() -> Result<()> {
  let manifest = load("./tests/manifests/v0/negative/multiple-inputs.wafl")?;
  let result = from_manifest(&manifest);
  assert!(result.is_err());

  Ok(())
}
