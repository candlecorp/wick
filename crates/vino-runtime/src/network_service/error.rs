use thiserror::Error;

use crate::dev::prelude::*;
#[derive(Error, Debug)]
pub enum NetworkError {
  #[error("Network not started")]
  NotStarted,
  #[error("Schematic {0} not found")]
  SchematicNotFound(String),
  #[error("Error initializing: {0}")]
  InitFailure(String),
  #[error("Error initializing: {}", join(.0, ", "))]
  SchematicInitFailure(Vec<SchematicError>),
  #[error("Maximum number of tries reached when resolving internal schematic references")]
  MaxTriesReached,
  #[error("Schematic Error: {0}")]
  SchematicError(String),
  #[error(transparent)]
  ProviderError(#[from] ProviderError),
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
  #[error("{0}")]
  ModelError(String),
  #[error("{0}")]
  ValidationError(String),
  #[error("{0}")]
  UnresolvableNetwork(String),
  #[error("Unknown provider '{0}'")]
  UnknownProvider(String),
  #[error("Invalid recipient '{0}'")]
  InvalidRecipient(String),
  #[error("Invalid state: {0}")]
  InvalidState(String),
}

impl From<NetworkModelError> for NetworkError {
  fn from(e: NetworkModelError) -> Self {
    NetworkError::ModelError(e.to_string())
  }
}

impl From<NetworkValidationError> for NetworkError {
  fn from(e: NetworkValidationError) -> Self {
    NetworkError::ValidationError(e.to_string())
  }
}

impl From<SchematicError> for NetworkError {
  fn from(e: SchematicError) -> Self {
    NetworkError::SchematicError(e.to_string())
  }
}

impl From<vino_loader::Error> for NetworkError {
  fn from(e: vino_loader::Error) -> Self {
    NetworkError::CommonError(CommonError::Loading(e.to_string()))
  }
}

impl From<vino_manifest::Error> for NetworkError {
  fn from(e: vino_manifest::Error) -> Self {
    NetworkError::CommonError(CommonError::Manifest(e.to_string()))
  }
}
