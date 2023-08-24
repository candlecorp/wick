use std::collections::HashMap;

use anyhow::Result;
use wick_config::config::{AppConfiguration, UninitializedConfiguration};
use wick_config::WickConfiguration;
use wick_packet::RuntimeConfig;

pub async fn load_config(path: &str) -> Result<UninitializedConfiguration> {
  let cargo_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let path = cargo_dir.join("../../bins/wick/tests").join(path);
  let config = WickConfiguration::fetch(&path, Default::default()).await?;
  Ok(config)
}

#[allow(unused)]
pub async fn load_wick_config(
  path: &str,
  root_config: Option<RuntimeConfig>,
  env: Option<HashMap<String, String>>,
) -> Result<WickConfiguration> {
  let mut config = load_config(path).await?;
  config.set_env(env).set_root_config(root_config);
  Ok(config.finish()?)
}

pub async fn load_app_config(path: &str, root_config: Option<RuntimeConfig>) -> Result<AppConfiguration> {
  let mut config = load_config(path).await?;
  let env = std::env::vars().collect();
  config.set_env(Some(env)).set_root_config(root_config);
  Ok(config.finish()?.try_app_config()?)
}
