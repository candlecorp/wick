use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tracing::{debug, Span};
use wick_config::config::{AppConfiguration, BoundIdentifier};
use wick_config::WickConfiguration;
use wick_trigger::resources::Resource;

async fn load_app_yaml(path: &str) -> anyhow::Result<AppConfiguration> {
  let mut config = WickConfiguration::fetch(path, Default::default()).await?;
  config.set_env(Some(std::env::vars().collect()));

  Ok(config.finish()?.try_app_config()?)
}

fn init_resources(config: &AppConfiguration) -> Result<HashMap<BoundIdentifier, Resource>> {
  let mut resources = HashMap::new();
  for res in config.resources() {
    let resource = Resource::new(res.kind().clone())?;
    resources.insert(res.binding().clone(), resource);
  }
  Ok(resources)
}

#[test_logger::test(tokio::test)]
async fn basic_cli() -> Result<()> {
  let manifest = load_app_yaml("./tests/manifests/v1/app_config/basic.yaml").await?;
  let rt = wick_trigger::build_trigger_runtime(&manifest, Span::current())?
    .build(None)
    .await?;
  let resources = Arc::new(init_resources(&manifest)?);

  let trigger_config = &manifest.triggers()[0];
  debug!(?trigger_config, "loading trigger");
  let config = trigger_config.clone();
  let name = manifest.name().to_owned();
  let app_config = manifest.clone();
  let trigger = wick_host::triggers::load_trigger(&trigger_config.kind())?;

  let inner = trigger.clone();
  let resources = resources.clone();
  inner
    .run(name, rt, app_config, config, resources.clone(), Span::current())
    .await?;
  Ok(())
}

mod integration_test {
  use tracing::Span;

  use super::*;
  #[test_logger::test(tokio::test)]
  async fn cli_with_db() -> Result<()> {
    let manifest = load_app_yaml("../../../examples/cli/wasm-calling-postgres.wick").await?;
    let rt = wick_trigger::build_trigger_runtime(&manifest, Span::current())?
      .build(None)
      .await?;
    let resources = Arc::new(init_resources(&manifest)?);

    let trigger_config = &manifest.triggers()[0];
    debug!(?trigger_config, "loading trigger");
    let config = trigger_config.clone();
    let name = manifest.name().to_owned();
    let app_config = manifest.clone();
    let trigger = wick_host::triggers::load_trigger(&trigger_config.kind())?;

    let inner = trigger.clone();
    let resources = resources.clone();
    let task = tokio::spawn(async move {
      let _ = inner
        .run(name, rt, app_config, config, resources.clone(), Span::current())
        .await;
      inner.wait_for_done().await;
    });
    let fut = tokio::time::timeout(Duration::from_millis(10000), task);
    println!("waiting for trigger to finish...");
    let _ = fut.await?;
    println!("trigger finished");

    Ok(())
  }
}
