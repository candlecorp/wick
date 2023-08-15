use std::convert::Infallible;

use thiserror::Error;
use wick_config::config::{ComponentKind, TriggerKind};

pub use crate::components::error::ComponentError;
use crate::resources::ResourceKind;
pub use crate::runtime_service::error::EngineError;

#[derive(Debug, Clone, Copy)]
pub enum Context {
  Trigger,
  TriggerKind(TriggerKind),
  Import,
  Resource,
  ResourceKind(ResourceKind),
  Component,
  ComponentKind(ComponentKind),
}

impl std::fmt::Display for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Context::Trigger => write!(f, "trigger"),
      Context::TriggerKind(kind) => write!(f, "trigger {}", kind),
      Context::Import => write!(f, "import"),
      Context::Resource => write!(f, "resource"),
      Context::ResourceKind(kind) => write!(f, "resource {}", kind),
      Context::Component => write!(f, "component"),
      Context::ComponentKind(kind) => write!(f, "component {}", kind),
    }
  }
}

impl From<TriggerKind> for Context {
  fn from(kind: TriggerKind) -> Self {
    Context::TriggerKind(kind)
  }
}

impl From<ResourceKind> for Context {
  fn from(kind: ResourceKind) -> Self {
    Context::ResourceKind(kind)
  }
}

impl From<ComponentKind> for Context {
  fn from(kind: ComponentKind) -> Self {
    Context::ComponentKind(kind)
  }
}

#[derive(Error, Debug)]
pub enum RuntimeError {
  #[error("Invalid {0} configuration, expected configuration for {0}")]
  InvalidConfig(Context, TriggerKind),

  #[error("{0} requested resource '{1}' which could not be found")]
  ResourceNotFound(Context, String),

  #[error("{0} requires resource kind {1}, not {2}")]
  InvalidResourceType(Context, ResourceKind, ResourceKind),

  #[error("{0} did not shutdown gracefully: {1}")]
  ShutdownFailed(Context, String),

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

  #[error("Could not render dotviz: {0}")]
  DotViz(flow_graph_interpreter::error::InterpreterError),
}

impl From<Infallible> for RuntimeError {
  fn from(_: Infallible) -> Self {
    unreachable!()
  }
}
