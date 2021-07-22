use anyhow::Result;

#[test_env_log::test(tokio::test)]
async fn test_api() -> Result<()> {
  test_interfaces_authentication::test::test_api(
    vino_authentication_inmemory::provider::Provider::default(),
  )
  .await?;

  Ok(())
}
