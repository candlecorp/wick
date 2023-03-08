use thiserror::Error;

use crate::dev::prelude::*;
use crate::BoxError;

#[derive(Error, Debug)]
pub enum CollectionError {
  #[error("Component not found: {0}")]
  ComponentNotFound(String),
  #[error("{0}")]
  NetworkError(String),

  #[error("{0}")]
  Mesh(String),
  #[error("Error initializing subnetwork '{0}' : {1}")]
  SubNetwork(String, String),

  #[error("{0}")]
  Downstream(Box<dyn std::error::Error + Send + Sync>),

  #[error(transparent)]
  InvocationError(#[from] InvocationError),

  #[error(transparent)]
  LoadFailed(#[from] wick_loader_utils::Error),
  #[error(transparent)]
  RpcHandlerError(#[from] Box<wick_rpc::Error>),
}

// impl From<wick_component_grpctar::Error> for CollectionError {
//   fn from(e: wick_component_grpctar::Error) -> Self {
//     CollectionError::Downstream(Box::new(e))
//   }
// }

// impl From<wick_component_grpc::Error> for CollectionError {
//   fn from(e: wick_component_grpc::Error) -> Self {
//     CollectionError::Downstream(Box::new(e))
//   }
// }

impl From<wick_component_wasm::Error> for CollectionError {
  fn from(e: wick_component_wasm::Error) -> Self {
    CollectionError::Downstream(Box::new(e))
  }
}

// impl From<wick_component_nats::error::MeshError> for CollectionError {
//   fn from(e: wick_component_nats::error::MeshError) -> Self {
//     CollectionError::Mesh(e.to_string())
//   }
// }

impl From<BoxError> for CollectionError {
  fn from(e: BoxError) -> Self {
    CollectionError::Mesh(e.to_string())
  }
}
