use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use async_trait::async_trait;
use structured_output::StructuredOutput;
use tracing::Span;
use wick_config::config::{AppConfiguration, TriggerDefinition};
use wick_runtime::{Runtime, RuntimeBuilder, RuntimeConstraint};

use crate::error::Error;
use crate::resources::Resource;

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
    runtime: Runtime,
    app_config: AppConfiguration,
    config: TriggerDefinition,
    resources: Arc<HashMap<String, Resource>>,
    span: Span,
  ) -> Result<StructuredOutput, Error>;

  /// Shutdown a running trigger.
  async fn shutdown_gracefully(self) -> Result<(), Error>;

  /// Wait for the trigger to finish.
  #[must_use = "this returns the output of the trigger"]
  async fn wait_for_done(&self) -> StructuredOutput;
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
