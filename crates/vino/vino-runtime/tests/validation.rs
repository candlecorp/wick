use runtime_testutils::*;
type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn missing_provider() -> Result<()> {
  let result = init_network_from_yaml("./manifests/v0/validation/missing-provider.yaml").await;

  assert!(result.is_err());
  Ok(())
}
