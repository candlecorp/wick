use thiserror::Error;

use crate::dev::prelude::*;
#[derive(Error, Debug)]
pub enum NetworkError {
  #[error(transparent)]
  SchematicGraph(#[from] wasmflow_schematic_graph::error::Error),
  #[error("Could not start interpreter from '{0}': {1}")]
  InterpreterInit(String, wasmflow_interpreter::InterpreterError),
  #[error(transparent)]
  Loading(#[from] wasmflow_loader::Error),
  #[error(transparent)]
  Manifest(#[from] wasmflow_manifest::Error),

  // OLD
  #[error(transparent)]
  ProviderError(#[from] ProviderError),

  #[error(transparent)]
  RpcHandlerError(#[from] Box<wasmflow_rpc::Error>),

  #[error("Request timeout out")]
  Timeout,
}

impl From<NetworkError> for ProviderError {
  fn from(e: NetworkError) -> Self {
    ProviderError::NetworkError(e.to_string())
  }
}
