use thiserror::Error;

use crate::dev::prelude::*;
#[derive(Error, Debug)]
pub enum NetworkError {
  #[error(transparent)]
  SchematicGraph(#[from] vino_schematic_graph::error::Error),
  #[error(transparent)]
  Interpreter(#[from] vino_interpreter::InterpreterError),
  #[error(transparent)]
  Loading(#[from] vino_loader::Error),
  #[error(transparent)]
  Manifest(#[from] vino_manifest::Error),

  // OLD
  #[error(transparent)]
  ProviderError(#[from] ProviderError),

  #[error(transparent)]
  RpcHandlerError(#[from] Box<vino_rpc::Error>),

  #[error("Request timeout out")]
  Timeout,
}

impl From<NetworkError> for ProviderError {
  fn from(e: NetworkError) -> Self {
    ProviderError::NetworkError(e.to_string())
  }
}
