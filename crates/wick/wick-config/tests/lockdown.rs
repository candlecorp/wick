mod utils;
mod integration_test {

  use anyhow::Result;
  use wick_config::lockdown::Lockdown;

  use crate::utils::load;

  #[test_logger::test(tokio::test)]
  async fn test_lockdown() -> Result<()> {
    let component = load("./tests/manifests/v1/component-resources.yaml")
      .await?
      .try_component_config()?;
    let lockdown = load("./tests/manifests/v1/lockdown.yaml")
      .await?
      .try_lockdown_config()?;
    component.lockdown(Some("test_id"), &lockdown)?;

    Ok(())
  }
}
