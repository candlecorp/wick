use std::collections::HashMap;
use std::sync::Arc;
mod cli;

use async_trait::async_trait;
use cli::Cli;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use wick_config::{TriggerDefinition, TriggerKind};

use crate::dev::prelude::RuntimeError;

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ApplicationConfig {
//   pub name: String,
//   pub version: String,
//   pub dependencies: Option<HashMap<String, String>>,
//   pub channels: HashMap<String, PluginConfiguration>,
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct PluginConfiguration {
//   pub uses: String,
//   pub with: serde_value::Value,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct ApplicationContext {
//   pub name: String,
//   pub version: String,
//   pub inherent_data: Option<InherentData>,
// }

#[async_trait]
pub trait Trigger {
  async fn run(&self, name: String, config: TriggerDefinition) -> Result<(), RuntimeError>;
  async fn shutdown_gracefully(&mut self) -> Result<(), RuntimeError>;
}

pub(crate) type TriggerLoader = Arc<dyn Fn() -> Result<Box<dyn Trigger + Send + Sync>, RuntimeError> + Send + Sync>;

static TRIGGER_LOADER_REGISTRY: Lazy<Mutex<HashMap<TriggerKind, TriggerLoader>>> = Lazy::new(|| {
  let mut m: HashMap<TriggerKind, TriggerLoader> = HashMap::new();
  m.insert(TriggerKind::Cli, Arc::new(Cli::load));
  Mutex::new(m)
});

#[must_use]
pub fn get_trigger_loader(name: &TriggerKind) -> Option<TriggerLoader> {
  TRIGGER_LOADER_REGISTRY.lock().get(name).cloned()
}
