use wick_packet::Entity;

use super::components::core_component::OpInitError;
use super::executor::error::ExecutionError;
use super::program::validator::error::{OperationInvalid, ValidationError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error(transparent)]
  ExecutionError(#[from] ExecutionError),

  #[error("{}", .0.iter().map(|e|e.to_string()).collect::<Vec<_>>().join(", "))]
  ValidationError(Vec<OperationInvalid>),

  #[error("Early error: {:?}", .0)]
  EarlyError(ValidationError),

  #[error("Could not find operation '{}' ({0}). Known operations are: {}",.0.operation_id(), .1.join(", "))]
  OpNotFound(Entity, Vec<String>),

  #[error("Could not find component '{}' ({0}). Namespaces handled by this resource are: {}", .0.component_id(), .1.join(", "))]
  TargetNotFound(Entity, Vec<String>),

  #[error("Error shutting down component: {0}")]
  ComponentShutdown(String),

  #[error("Shutdown failed: {0}")]
  Shutdown(String),

  #[error("Event loop panicked: {0}")]
  EventLoopPanic(String),

  #[error("Shutdown timed out")]
  ShutdownTimeout,

  #[error("Namespace '{0}' already exists, can not overwrite")]
  DuplicateNamespace(String),

  #[error(transparent)]
  OperationInit(#[from] OpInitError),

  #[error("Interpreter can only expose one component at a time, found multiple: {}", .0.join(", "))]
  ExposedLimit(Vec<String>),

  #[error("Could not render operation configuration: {0}")]
  Configuration(String),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum StateError {
  #[error("Payload for port '{0}' missing from input stream")]
  PayloadMissing(String),
  #[error(
    "Could not find port named '{0}'. This can result from providing more input than a schematic has ports for."
  )]
  MissingPortName(String),
  #[error("Attempted to access nonexistant component '{0}'")]
  MissingComponent(String),
  #[error("Tried to decrement pending counter for non-existent or zero ID.")]
  TooManyComplete,
}

impl From<Vec<OperationInvalid>> for Error {
  fn from(v: Vec<OperationInvalid>) -> Self {
    Error::ValidationError(v)
  }
}
