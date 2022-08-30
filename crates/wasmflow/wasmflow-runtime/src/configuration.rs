use std::{collections::HashMap, fmt, env, sync::Arc};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Serialize, Deserialize};
use serde_yaml;
use serde_value;
use anyhow::{Result, Context};

use crate::dev::prelude::RuntimeError;

use super::cli::CLI;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationConfig {
  pub name: String,
  pub version: String,
  pub dependencies: Option<HashMap<String, String>>,
  pub channels: HashMap<String, PluginConfiguration>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfiguration {
  pub uses: String,
  pub with: serde_value::Value,
}

pub fn load_yaml(filename: &String) -> Result<ApplicationConfig> {
  let f = std::fs::File::open(filename)?;
  let config: ApplicationConfig = serde_yaml::from_reader(f)?;
  Ok(config)
}

pub fn from_reader<R>(reader: R) -> Result<ApplicationConfig>
where
    R: std::io::Read {
  let config: ApplicationConfig = serde_yaml::from_reader(reader)?;
  Ok(config)
}

pub fn from_string(str: &String) -> Result<ApplicationConfig> {
  let config: ApplicationConfig = serde_yaml::from_str(str)?;
  Ok(config)
}

#[async_trait]
pub trait Channel{
  async fn run(&self) -> Result<()>;
  async fn shutdown_gracefully(&self) -> Result<()>;
}

pub type ChannelLoader = Arc<dyn Fn(serde_value::Value) -> Result<Box<dyn Channel + Send + Sync>> + Send + Sync>;

static CHANNEL_LOADER_REGISTRY: Lazy<Mutex<HashMap<String, ChannelLoader>>> = Lazy::new(|| {
  let mut m: HashMap<String, ChannelLoader> = HashMap::new();
  m.insert("channels.wasmflow.cli@v1".to_owned(), Arc::new(CLI::load));
  Mutex::new(m)
});

pub fn get_channel_loader(name: &str) -> Option<ChannelLoader> {
  CHANNEL_LOADER_REGISTRY.lock().get(name).cloned()
}

pub fn register_channel_loader(name: &str, loader: ChannelLoader) {
  CHANNEL_LOADER_REGISTRY.lock().insert(name.to_owned(), loader);
}
