use thiserror::Error;
use wasmflow_interpreter::BoxError;

use crate::dev::prelude::*;

#[derive(Error, Debug)]
pub enum ProviderError {
  #[error("Component not found: {0}")]
  ComponentNotFound(String),
  #[error("{0}")]
  NetworkError(String),

  #[error("{0}")]
  Lattice(String),
  #[error("Error initializing subnetwork '{0}' : {1}")]
  SubNetwork(String, String),

  #[error(transparent)]
  ParProviderError(#[from] wasmflow_collection_par::Error),
  #[error(transparent)]
  GrpcProviderError(#[from] wasmflow_collection_grpc::Error),
  #[error(transparent)]
  WasmProviderError(#[from] wasmflow_collection_wasm::Error),
  #[error(transparent)]
  InvocationError(#[from] InvocationError),
  #[error(transparent)]
  TransportError(#[from] wasmflow_transport::error::TransportError),
  #[error(transparent)]
  LoadFailed(#[from] wasmflow_loader::Error),
  #[error(transparent)]
  RpcHandlerError(#[from] Box<wasmflow_rpc::Error>),
}

impl From<wasmflow_collection_nats::error::LatticeProviderError> for ProviderError {
  fn from(e: wasmflow_collection_nats::error::LatticeProviderError) -> Self {
    ProviderError::Lattice(e.to_string())
  }
}

impl From<BoxError> for ProviderError {
  fn from(e: BoxError) -> Self {
    ProviderError::Lattice(e.to_string())
  }
}
