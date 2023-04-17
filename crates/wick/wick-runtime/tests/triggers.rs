use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
mod utils;
use tracing::debug;
use wick_config::config::AppConfiguration;
use wick_config::WickConfiguration;
use wick_runtime::resources::Resource;

pub async fn load_app_yaml(path: &str) -> anyhow::Result<AppConfiguration> {
  let host_def = WickConfiguration::load_from_file(path).await?.try_app_config()?;
  Ok(host_def)
}

fn init_resources(config: &AppConfiguration) -> Result<HashMap<String, Resource>> {
  let mut resources = HashMap::new();
  for (id, def) in config.resources() {
    let resource = Resource::new(def.kind.clone())?;
    resources.insert(id.clone(), resource);
  }
  Ok(resources)
}

#[test_logger::test(tokio::test)]
async fn basic_cli() -> Result<()> {
  let manifest = load_app_yaml("./manifests/v1/app_config/basic.yaml").await?;
  let resources = Arc::new(init_resources(&manifest)?);

  let trigger_config = &manifest.triggers()[0];
  debug!(?trigger_config, "loading trigger");
  let config = trigger_config.clone();
  let name = manifest.name();
  let app_config = manifest.clone();

  match wick_runtime::get_trigger_loader(&trigger_config.kind()) {
    Some(loader) => {
      let loader = loader()?;
      let inner = loader.clone();
      let resources = resources.clone();
      inner.run(name, app_config, config, resources.clone()).await?;
    }
    _ => {
      panic!("could not find trigger {}", &trigger_config.kind());
    }
  };
  Ok(())
}

mod integration_test {
  use super::*;
  #[test_logger::test(tokio::test)]
  async fn cli_with_db() -> Result<()> {
    // let manifest = load_app_yaml("./manifests/v1/app_config/postgres.yaml").await?;
    let manifest = load_app_yaml("../../../examples/postgres.yaml").await?;
    let resources = Arc::new(init_resources(&manifest)?);

    let trigger_config = &manifest.triggers()[0];
    debug!(?trigger_config, "loading trigger");
    let config = trigger_config.clone();
    let name = manifest.name();
    let app_config = manifest.clone();

    let task = match wick_runtime::get_trigger_loader(&trigger_config.kind()) {
      Some(loader) => {
        let loader = loader()?;
        let inner = loader.clone();
        let resources = resources.clone();
        tokio::spawn(async move {
          let _ = inner.run(name, app_config, config, resources.clone()).await;
          inner.wait_for_done().await;
        })
      }
      _ => {
        panic!("could not find trigger {}", &trigger_config.kind());
      }
    };
    let fut = tokio::time::timeout(Duration::from_millis(5000), task);
    println!("waiting for trigger to finish...");
    let _ = fut.await?;
    println!("trigger finished");

    Ok(())
  }
}
