use anyhow::Result;

#[test_env_log::test(tokio::test)]
async fn test_api() -> Result<()> {
  test_interfaces_collection::test::test_api(vino_collection_fs::provider::Provider::new(
    std::env::temp_dir(),
  ))
  .await?;

  Ok(())
}
