use anyhow::Result;

#[test_logger::test(tokio::test)]
async fn test_api() -> Result<()> {
  test_interface_collection::test::test_api(Box::new(vino_collection_fs::provider::Provider::new(
    std::env::temp_dir(),
  )))
  .await?;

  Ok(())
}
