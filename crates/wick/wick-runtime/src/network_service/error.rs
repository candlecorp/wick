use thiserror::Error;

use crate::dev::prelude::*;
#[derive(Error, Debug)]
pub enum NetworkError {
  #[error(transparent)]
  SchematicGraph(#[from] flow_graph::error::Error),
  #[error("Could not start interpreter from '{0}': {1}")]
  InterpreterInit(String, flow_graph_interpreter::error::InterpreterError),
  #[error(transparent)]
  Loading(#[from] wick_loader_utils::Error),
  #[error(transparent)]
  Manifest(#[from] wick_config_component::Error),

  // OLD
  #[error(transparent)]
  CollectionError(#[from] CollectionError),

  #[error(transparent)]
  RpcHandlerError(#[from] Box<wick_rpc::Error>),

  #[error("Request timeout out")]
  Timeout,
}

impl From<NetworkError> for CollectionError {
  fn from(e: NetworkError) -> Self {
    CollectionError::NetworkError(e.to_string())
  }
}
