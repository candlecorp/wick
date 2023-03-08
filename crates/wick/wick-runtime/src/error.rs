use thiserror::Error;
use wick_config::TriggerKind;

pub use crate::collections::error::CollectionError;
pub use crate::network_service::error::NetworkError;

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

  #[error(transparent)]
  ComponentError(#[from] CollectionError),
  #[error(transparent)]
  NetworkError(#[from] NetworkError),

  #[error("{0}")]
  Serialization(String),
  #[error("{0}")]
  InitializationFailed(String),
}
