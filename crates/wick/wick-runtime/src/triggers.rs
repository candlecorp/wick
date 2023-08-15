use std::convert::Infallible;
mod cli;
mod http;
mod time;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use structured_output::StructuredOutput;
use tracing::Span;
use wick_config::config::{
  ComponentDefinition,
  ComponentOperationExpression,
  ImportBinding,
  TriggerDefinition,
  TriggerKind,
};
use wick_packet::Entity;

use crate::dev::prelude::*;
use crate::resources::Resource;
use crate::RuntimeBuilder;

fn build_trigger_runtime(config: &AppConfiguration, span: Span) -> Result<RuntimeBuilder, Infallible> {
  let mut rt = RuntimeBuilder::default();
  if let Some(fetch_opts) = config.options() {
    rt = rt.allow_latest(*fetch_opts.allow_latest());
    rt = rt.allowed_insecure(fetch_opts.allow_insecure().clone());
  }
  for import in config.imports() {
    rt.add_import(import.clone());
  }
  for resource in config.resources() {
    rt.add_resource(resource.clone());
  }
  rt = rt.span(span);
  Ok(rt)
}

#[async_trait]
pub trait Trigger {
  async fn run(
    &self,
    name: String,
    app_config: AppConfiguration,
    config: TriggerDefinition,
    resources: Arc<HashMap<String, Resource>>,
    span: Span,
  ) -> Result<StructuredOutput, RuntimeError>;
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

pub(crate) fn resolve_or_import_component(
  app_config: &AppConfiguration,
  optional_name: impl AsRef<str>,
  operation: &ComponentOperationExpression,
) -> Result<(Entity, Option<ImportBinding>), RuntimeError> {
  if let ComponentDefinition::Reference(cref) = operation.component() {
    let _assert = app_config
      .resolve_binding(cref.id())
      .map_err(|e| {
        RuntimeError::InitializationFailed(format!("Error initializing component {}: error was {}", cref.id(), e))
      })?
      .try_component()
      .map_err(|e| RuntimeError::ReferenceError(cref.id().to_owned(), e))?;
    Ok((Entity::operation(cref.id(), operation.name()), None))
  } else {
    Ok((
      Entity::operation(&optional_name, operation.name()),
      Some(ImportBinding::component(optional_name, operation.component().clone())),
    ))
  }
}

pub(crate) fn resolve_ref(
  app_config: &AppConfiguration,
  component: &ComponentDefinition,
) -> Result<ComponentDefinition, RuntimeError> {
  let def = if let ComponentDefinition::Reference(cref) = component {
    app_config
      .resolve_binding(cref.id())
      .map_err(|e| {
        RuntimeError::InitializationFailed(format!("Error initializing component {}: error was {}", cref.id(), e))
      })?
      .try_component()
      .map_err(|e| RuntimeError::ReferenceError(cref.id().to_owned(), e))?
  } else {
    component.clone()
  };
  Ok(def)
}
