use thiserror::Error;

use crate::dev::prelude::*;
#[derive(Error, Debug)]
pub enum NetworkError {
  #[error("Network not started")]
  NotStarted,
  #[error("Schematic {0} not found")]
  SchematicNotFound(String),
  #[error("Error initializing: {}", join(.0, ", "))]
  InitializationError(Vec<SchematicError>),
  #[error("Maximum number of tries reached when resolving internal schematic references")]
  MaxTriesReached,
  #[error(transparent)]
  SchematicError(#[from] Box<SchematicError>),
  #[error(transparent)]
  ComponentError(#[from] ProviderError),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error("Error executing request: {0}")]
  ExecutionError(String),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  RpcHandlerError(#[from] Box<vino_rpc::Error>),
  #[error("Lattice error: {0}")]
  Lattice(String),
}

impl From<vino_lattice::Error> for NetworkError {
  fn from(e: vino_lattice::Error) -> Self {
    NetworkError::Lattice(e.to_string())
  }
}
