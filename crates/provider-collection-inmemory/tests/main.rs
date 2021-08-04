use anyhow::Result;

#[test_env_log::test(tokio::test)]
async fn test_api() -> Result<()> {
  test_interfaces_collection::test::test_api(Box::new(
    vino_collection_inmemory::provider::Provider::default(),
  ))
  .await?;

  Ok(())
}
