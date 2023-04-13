use std::collections::HashMap;
use std::sync::Arc;
mod cli;
mod http;
mod time;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use wick_config::config::{AppConfiguration, ComponentDefinition, TriggerDefinition, TriggerKind};

use crate::dev::prelude::*;
use crate::resources::Resource;

#[async_trait]
pub trait Trigger {
  async fn run(
    &self,
    name: String,
    app_config: AppConfiguration,
    config: TriggerDefinition,
    resources: Arc<HashMap<String, Resource>>,
  ) -> Result<(), RuntimeError>;
  async fn shutdown_gracefully(self) -> Result<(), RuntimeError>;
  async fn wait_for_done(&self);
}

pub(crate) type TriggerLoader = Arc<dyn Fn() -> Result<Arc<dyn Trigger + Send + Sync>, RuntimeError> + Send + Sync>;

static TRIGGER_LOADER_REGISTRY: Lazy<Mutex<HashMap<TriggerKind, TriggerLoader>>> = Lazy::new(|| {
  let mut m: HashMap<TriggerKind, TriggerLoader> = HashMap::new();
  m.insert(TriggerKind::Cli, Arc::new(cli::Cli::load));
  m.insert(TriggerKind::Http, Arc::new(http::Http::load));
  m.insert(TriggerKind::Time, Arc::new(time::Time::load));
  Mutex::new(m)
});

#[must_use]
pub fn get_trigger_loader(name: &TriggerKind) -> Option<TriggerLoader> {
  TRIGGER_LOADER_REGISTRY.lock().get(name).cloned()
}

pub(crate) fn resolve_ref(
  app_config: &AppConfiguration,
  component: &ComponentDefinition,
) -> Result<ComponentDefinition, RuntimeError> {
  let def = if let ComponentDefinition::Reference(cref) = component {
    app_config
      .resolve_binding(cref.id())
      .ok_or_else(|| {
        RuntimeError::InitializationFailed(format!("Could not find a component by the name of {}", cref.id()))
      })?
      .component()
      .map_err(|e| RuntimeError::ReferenceError(cref.id().to_owned(), e))?
      .clone()
  } else {
    component.clone()
  };
  Ok(def)
}
