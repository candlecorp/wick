use std::convert::Infallible;

use thiserror::Error;
use wick_config::config::TriggerKind;

pub use crate::components::error::ComponentError;
use crate::resources::ResourceKind;
pub use crate::runtime_service::error::EngineError;

#[derive(Error, Debug)]
pub enum RuntimeError {
  #[error("Invalid trigger configuration, expected configuration for {0}")]
  InvalidTriggerConfig(TriggerKind),

  #[error("Trigger {0} requested resource '{1}' which could not be found")]
  ResourceNotFound(TriggerKind, String),

  #[error("Trigger {0} referenced import '{1}' which could not be found")]
  NotFound(String, String),

  #[error("Trigger {0} requires resource kind {1}, not {2}")]
  InvalidResourceType(TriggerKind, ResourceKind, ResourceKind),

  #[error("Trigger {0} did not shutdown gracefully: {1}")]
  ShutdownFailed(TriggerKind, String),

  #[error("The supplied id '{0}' does not point to a correct type or valid type: {1}")]
  ReferenceError(String, wick_config::Error),

  #[error("{0}")]
  InitializationFailed(String),

  #[error("Invocation error: {0}")]
  InvocationError(String),

  #[error(transparent)]
  ComponentError(#[from] ComponentError),

  #[error("Request timeout out")]
  Timeout,

  #[error("Error starting schedule: {0}")]
  ScheduleStartError(String),

  #[error("Could not render configuration: {0}")]
  Configuration(String),

  #[error("Runtime can not be built without a wick configuration")]
  MissingComponentDefinition,
}

impl From<Infallible> for RuntimeError {
  fn from(_: Infallible) -> Self {
    unreachable!()
  }
}
