use std::convert::Infallible;
mod cli;
mod http;
mod time;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use structured_output::StructuredOutput;
use tracing::Span;
use wick_config::config::{ComponentDefinition, ComponentOperationExpression, TriggerDefinition, TriggerKind};

use crate::dev::prelude::*;
use crate::resources::Resource;
use crate::runtime::RuntimeConstraint;
use crate::RuntimeBuilder;

pub fn build_trigger_runtime(config: &AppConfiguration, span: Span) -> Result<RuntimeBuilder, Infallible> {
  let mut rt = RuntimeBuilder::default();
  if let Some(fetch_opts) = config.options() {
    rt = rt.allow_latest(*fetch_opts.allow_latest());
    rt = rt.allowed_insecure(fetch_opts.allow_insecure().clone());
  }
  for import in config.import() {
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
  /// Start executing the trigger.
  async fn run(
    &self,
    name: String,
    runtime: crate::Runtime,
    app_config: AppConfiguration,
    config: TriggerDefinition,
    resources: Arc<HashMap<String, Resource>>,
    span: Span,
  ) -> Result<StructuredOutput, RuntimeError>;

  /// Shutdown a running trigger.
  async fn shutdown_gracefully(self) -> Result<(), RuntimeError>;

  /// Wait for the trigger to finish.
  async fn wait_for_done(&self);
}

/// Runtime configuration necessary for a trigger to execute.
#[derive(Debug, Clone)]
pub struct TriggerRuntimeConfig {
  pub(crate) constraints: Vec<RuntimeConstraint>,
}

impl TriggerRuntimeConfig {
  /// Extend a runtime builder with the configuration contained within.
  pub fn extend_runtime(self, rt: &mut RuntimeBuilder) {
    for constraint in self.constraints {
      rt.add_constraint(constraint);
    }
  }
}

pub(crate) trait ComponentId {
  fn component_id(&self) -> Result<&str, RuntimeError>;
}

impl ComponentId for ComponentOperationExpression {
  fn component_id(&self) -> Result<&str, RuntimeError> {
    match self.component() {
      ComponentDefinition::Reference(r) => Ok(r.id()),
      _ => Err(RuntimeError::InvalidConfig(
        Context::Trigger,
        "expected a component reference but found an unimported definition, this is a bug".to_owned(),
      )),
    }
  }
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
