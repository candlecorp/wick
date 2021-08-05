use anyhow::Result;

#[test_env_log::test(tokio::test)]
async fn test_api() -> Result<()> {
  test_interface_authentication::test::test_api(Box::new(
    vino_authentication_inmemory::provider::Provider::default(),
  ))
  .await?;

  Ok(())
}
