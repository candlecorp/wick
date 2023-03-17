use thiserror::Error;
use wick_config::TriggerKind;

pub use crate::components::error::ComponentError;
pub use crate::network_service::error::NetworkError;
use crate::resources::ResourceKind;

#[derive(Error, Debug, Clone, Copy)]
pub struct ConversionError(pub &'static str);

impl std::fmt::Display for ConversionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.0)
  }
}

#[derive(Error, Debug)]
#[error("Invocation error: {0}")]
pub struct InvocationError(pub String);

#[derive(Error, Debug)]
pub enum RuntimeError {
  #[error(transparent)]
  SchematicGraph(#[from] flow_graph::error::Error),

  #[error("Invalid trigger configuration, expected configuration for {0}")]
  InvalidTriggerConfig(TriggerKind),

  #[error(transparent)]
  InvocationError(#[from] InvocationError),

  #[error("Trigger {0} requested resource '{1}' which could not be found")]
  ResourceNotFound(TriggerKind, String),

  #[error("Trigger {0} requires resource kind {1}, not {2}")]
  InvalidResourceType(TriggerKind, ResourceKind, ResourceKind),

  #[error("Trigger {0} did not shutdown gracefully: {1}")]
  ShutdownFailed(TriggerKind, String),

  #[error("Trigger {0} died prematurely: {1}")]
  TriggerFailed(TriggerKind, String),

  #[error(transparent)]
  ComponentError(#[from] ComponentError),
  #[error(transparent)]
  NetworkError(#[from] NetworkError),

  #[error("The supplied id '{0}' does not point to the correct type: {1}")]
  ReferenceError(String, wick_config::error::ReferenceError),

  #[error("{0}")]
  Serialization(String),
  #[error("{0}")]
  InitializationFailed(String),
}
