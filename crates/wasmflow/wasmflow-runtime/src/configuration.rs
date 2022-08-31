use std::collections::HashMap;
use std::sync::Arc;
use std::{env, fmt};

use anyhow::{Context, Result};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use wasmflow_sdk::v1::InherentData;
use {serde_value, serde_yaml};

use super::cli::CLI;
use crate::dev::prelude::RuntimeError;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationContext {
  pub name: String,
  pub version: String,
  pub inherent_data: Option<InherentData>,
}

pub fn load_yaml(filename: &String) -> Result<ApplicationConfig> {
  let f = std::fs::File::open(filename)?;
  let config: ApplicationConfig = serde_yaml::from_reader(f)?;
  Ok(config)
}

pub fn from_reader<R>(reader: R) -> Result<ApplicationConfig>
where
  R: std::io::Read,
{
  let config: ApplicationConfig = serde_yaml::from_reader(reader)?;
  Ok(config)
}

pub fn from_string(str: &str) -> Result<ApplicationConfig> {
  let config: ApplicationConfig = serde_yaml::from_str(str)?;
  Ok(config)
}

#[async_trait]
pub trait Channel {
  async fn run(&self) -> Result<()>;
  async fn shutdown_gracefully(&self) -> Result<()>;
}

pub type ChannelLoader =
  Arc<dyn Fn(ApplicationContext, serde_value::Value) -> Result<Box<dyn Channel + Send + Sync>> + Send + Sync>;

static CHANNEL_LOADER_REGISTRY: Lazy<Mutex<HashMap<String, ChannelLoader>>> = Lazy::new(|| {
  let mut m: HashMap<String, ChannelLoader> = HashMap::new();
  m.insert("channels.wasmflow.cli@v1".to_owned(), Arc::new(CLI::load));
  Mutex::new(m)
});

#[must_use]
pub fn get_channel_loader(name: &str) -> Option<ChannelLoader> {
  CHANNEL_LOADER_REGISTRY.lock().get(name).cloned()
}

pub fn register_channel_loader(name: &str, loader: ChannelLoader) {
  CHANNEL_LOADER_REGISTRY.lock().insert(name.to_owned(), loader);
}
