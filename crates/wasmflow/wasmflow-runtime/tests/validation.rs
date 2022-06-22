use runtime_testutils::*;
type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn missing_collection() -> Result<()> {
  let result = init_network_from_yaml("./manifests/v0/validation/missing-collection.wafl").await;
  println!("result: {:?}", result);

  assert!(result.is_err());
  Ok(())
}
