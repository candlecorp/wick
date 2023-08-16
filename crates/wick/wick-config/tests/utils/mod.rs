#![allow(dead_code)]
use std::path::PathBuf;

use wick_config::config::{
  AppConfiguration,
  ComponentConfiguration,
  CompositeComponentImplementation,
  LockdownConfiguration,
  UninitializedConfiguration,
};
use wick_config::error::ManifestError;
use wick_config::WickConfiguration;
use wick_packet::RuntimeConfig;

pub async fn load_uninitialized(path: &str) -> Result<UninitializedConfiguration, ManifestError> {
  let path = PathBuf::from(path);
  let mut config = WickConfiguration::fetch(&path, Default::default()).await?;
  config.set_env(Some(std::env::vars().collect()));
  config.set_root_config(Some(RuntimeConfig::from([("component_config_name", "test".into())])));
  Ok(config)
}

pub async fn load(path: &str) -> Result<WickConfiguration, ManifestError> {
  load_uninitialized(path).await?.finish()
}

pub async fn load_app(path: &str) -> Result<AppConfiguration, ManifestError> {
  load(path).await?.try_app_config()
}

pub async fn load_component(path: &str) -> Result<ComponentConfiguration, ManifestError> {
  load(path).await?.try_component_config()
}

pub async fn load_composite(path: &str) -> Result<CompositeComponentImplementation, ManifestError> {
  Ok(load(path).await?.try_component_config()?.try_composite()?.clone())
}

pub async fn load_lockdown(path: &str) -> Result<LockdownConfiguration, ManifestError> {
  load(path).await?.try_lockdown_config()
}
