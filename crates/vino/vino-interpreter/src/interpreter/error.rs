use vino_schematic_graph::{ComponentIndex, PortReference};

use super::executor::error::ExecutionError;
use super::program::validator::error::ValidationError;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
  #[error(transparent)]
  InvalidState(#[from] StateError),
  #[error(transparent)]
  ExecutionError(#[from] ExecutionError),
  #[error("Validation errors: {:?}", .0)]
  ValidationError(Vec<ValidationError>),

  #[error("Shutdown failed: {0}")]
  ShutdownFailed(String),
}

pub(crate) fn missing_port<T: AsRef<PortReference>>(port: T) -> ExecutionError {
  ExecutionError::InvalidState(StateError::MissingPort(*port.as_ref()))
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum StateError {
  #[error("Payload for port '{0}' missing from input stream")]
  PayloadMissing(String),
  #[error("Attempted to access nonexistant port {:?}", 0)]
  MissingPort(PortReference),
  #[error("Attempted to access nonexistant component at index {0}")]
  MissingComponent(ComponentIndex),
  #[error("Attempted to access nonexistant provider '{0}'")]
  MissingProvider(String),
  #[error("{0}")]
  Other(String),
}

impl From<Vec<ValidationError>> for Error {
  fn from(v: Vec<ValidationError>) -> Self {
    Error::ValidationError(v)
  }
}
