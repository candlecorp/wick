use thiserror::Error;
use vino_interpreter::BoxError;

use crate::dev::prelude::*;

#[derive(Error, Debug)]
pub enum ProviderError {
  #[error("Component not found: {0}")]
  ComponentNotFound(String),
  #[error("{0}")]
  NetworkError(String),

  #[error("{0}")]
  Lattice(String),
  #[error("Error intializing subnetwork: {0}")]
  SubNetwork(String),

  #[error(transparent)]
  ParProviderError(#[from] vino_provider_par::Error),
  #[error(transparent)]
  GrpcProviderError(#[from] vino_provider_grpc::Error),
  #[error(transparent)]
  WasmProviderError(#[from] vino_provider_wasm::Error),
  #[error(transparent)]
  InvocationError(#[from] InvocationError),
  #[error(transparent)]
  TransportError(#[from] vino_transport::error::TransportError),
  #[error(transparent)]
  LoadFailed(#[from] vino_loader::Error),
  #[error(transparent)]
  RpcHandlerError(#[from] Box<vino_rpc::Error>),
}

impl From<vino_provider_lattice::error::LatticeProviderError> for ProviderError {
  fn from(e: vino_provider_lattice::error::LatticeProviderError) -> Self {
    ProviderError::Lattice(e.to_string())
  }
}

impl From<BoxError> for ProviderError {
  fn from(e: BoxError) -> Self {
    ProviderError::Lattice(e.to_string())
  }
}
