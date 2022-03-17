use vino_entity::Entity;

use super::executor::error::ExecutionError;
use super::program::validator::error::ValidationError;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
  #[error(transparent)]
  ExecutionError(#[from] ExecutionError),
  #[error("Validation errors: {:?}", .0)]
  ValidationError(Vec<ValidationError>),
  #[error("Early error: {:?}", .0)]
  EarlyError(ValidationError),
  #[error("Invalid target: {0}")]
  TargetNotFound(Entity),
  #[error("Shutdown failed: {0}")]
  ShutdownFailed(String),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum StateError {
  #[error("Payload for port '{0}' missing from input stream")]
  PayloadMissing(String),
  #[error("Could not find port named '{0}'")]
  MissingPortName(String),
  #[error("Attempted to access nonexistant provider '{0}'")]
  MissingProvider(String),
  #[error("Tried to decrement pending counter for non-existant or zero ID.")]
  TooManyComplete,
}

impl From<Vec<ValidationError>> for Error {
  fn from(v: Vec<ValidationError>) -> Self {
    Error::ValidationError(v)
  }
}

impl From<ValidationError> for Error {
  fn from(v: ValidationError) -> Self {
    Error::ValidationError(vec![v])
  }
}
