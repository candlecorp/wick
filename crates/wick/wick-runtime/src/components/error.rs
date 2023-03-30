use thiserror::Error;
use wick_config::config::LocationReference;

use crate::dev::prelude::*;
use crate::BoxError;

#[derive(Error, Debug)]
pub enum ComponentError {
  #[error("Component not found: {0}")]
  ComponentNotFound(String),
  #[error("{0}")]
  NetworkError(String),

  #[error("{0}")]
  Mesh(String),
  #[error("Error initializing subnetwork '{0}' : {1}")]
  SubNetwork(LocationReference, String),

  #[error("{0}")]
  Downstream(Box<dyn std::error::Error + Send + Sync>),

  #[error(transparent)]
  InvocationError(#[from] InvocationError),

  #[error(transparent)]
  RpcHandlerError(#[from] Box<wick_rpc::Error>),
}

impl From<wick_component_wasm::Error> for ComponentError {
  fn from(e: wick_component_wasm::Error) -> Self {
    ComponentError::Downstream(Box::new(e))
  }
}

// impl From<wick_component_nats::error::MeshError> for CollectionError {
//   fn from(e: wick_component_nats::error::MeshError) -> Self {
//     CollectionError::Mesh(e.to_string())
//   }
// }

impl From<BoxError> for ComponentError {
  fn from(e: BoxError) -> Self {
    ComponentError::Mesh(e.to_string())
  }
}
