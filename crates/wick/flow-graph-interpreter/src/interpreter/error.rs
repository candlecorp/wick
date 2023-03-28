use wick_packet::Entity;

use super::executor::error::ExecutionError;
use super::program::validator::error::{SchematicInvalid, ValidationError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error(transparent)]
  ExecutionError(#[from] ExecutionError),
  #[error("{}", .0.iter().map(|e|e.to_string()).collect::<Vec<_>>().join(", "))]
  ValidationError(Vec<SchematicInvalid>),
  #[error("Early error: {:?}", .0)]
  EarlyError(ValidationError),
  #[error("Could not find operation '{}' ({0}). Known flows are: {}",.0.name(), .1.join(", "))]
  SchematicNotFound(Entity, Vec<String>),
  #[error("Could not find target '{}' ({0}). Namespaces handled by this resource are: {}", .0.name(), .1.join(", "))]
  TargetNotFound(Entity, Vec<String>),
  #[error("Error shutting down collection: {0}")]
  CollectionShutdown(String),
  #[error("Shutdown failed: {0}")]
  Shutdown(String),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum StateError {
  #[error("Payload for port '{0}' missing from input stream")]
  PayloadMissing(String),
  #[error(
    "Could not find port named '{0}'. This can result from providing more input than a schematic has ports for."
  )]
  MissingPortName(String),
  #[error("Attempted to access nonexistant collection '{0}'")]
  MissingCollection(String),
  #[error("Tried to decrement pending counter for non-existent or zero ID.")]
  TooManyComplete,
}

impl From<Vec<SchematicInvalid>> for Error {
  fn from(v: Vec<SchematicInvalid>) -> Self {
    Error::ValidationError(v)
  }
}
