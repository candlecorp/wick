use thiserror::Error;
use wasmflow_interpreter::BoxError;

use crate::dev::prelude::*;

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
  LoadFailed(#[from] wasmflow_loader::Error),
  #[error(transparent)]
  RpcHandlerError(#[from] Box<wasmflow_rpc::Error>),
}

impl From<wasmflow_collection_par::Error> for CollectionError {
  fn from(e: wasmflow_collection_par::Error) -> Self {
    CollectionError::Downstream(Box::new(e))
  }
}

impl From<wasmflow_collection_grpc::Error> for CollectionError {
  fn from(e: wasmflow_collection_grpc::Error) -> Self {
    CollectionError::Downstream(Box::new(e))
  }
}

impl From<wasmflow_collection_wasm::Error> for CollectionError {
  fn from(e: wasmflow_collection_wasm::Error) -> Self {
    CollectionError::Downstream(Box::new(e))
  }
}

impl From<wasmflow_collection_nats::error::MeshError> for CollectionError {
  fn from(e: wasmflow_collection_nats::error::MeshError) -> Self {
    CollectionError::Mesh(e.to_string())
  }
}

impl From<BoxError> for CollectionError {
  fn from(e: BoxError) -> Self {
    CollectionError::Mesh(e.to_string())
  }
}
