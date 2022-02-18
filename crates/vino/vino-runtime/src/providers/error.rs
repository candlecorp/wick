use thiserror::Error;

use crate::dev::prelude::*;

#[derive(Error, Debug)]
pub enum ProviderError {
  #[error("Component not found: {0}")]
  ComponentNotFound(String),
  #[error("{0}")]
  NetworkError(String),
  #[error("{0}")]
  SchematicError(String),
  #[error("Invalid state: {0}")]
  InvalidState(String),
  #[error(transparent)]
  InvocationError(#[from] InvocationError),
  #[error("Provider uninitialized ({0})")]
  Uninitialized(i32),
  #[error(transparent)]
  ParProviderError(#[from] vino_provider_par::Error),
  #[error(transparent)]
  GrpcProviderError(#[from] vino_provider_grpc::Error),
  #[error(transparent)]
  WasmProviderError(#[from] vino_provider_wasm::Error),
  #[error("Failed to create a raw WebAssembly host")]
  WapcError,
  #[error(transparent)]
  ConversionError(#[from] ConversionError),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  LoadFailed(#[from] vino_loader::Error),
  #[error(transparent)]
  RpcError(#[from] vino_rpc::Error),
  #[error(transparent)]
  RpcHandlerError(#[from] Box<vino_rpc::Error>),
  #[error("Upstream RPC error: {0}")]
  RpcUpstreamError(String),
  #[error(transparent)]
  OutputError(#[from] vino_packet::error::DeserializationError),
  #[error(transparent)]
  RpcServerError(#[from] vino_invocation_server::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error("Grpc Provider error: {0}")]
  GrpcUrlProviderError(String),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
  #[error("{0}")]
  Lattice(String),
  #[error("Error intializing subnetwork: {0}")]
  SubNetwork(String),
  #[error("{0}")]
  StateUninitialized(String),
}

impl From<vino_provider_lattice::error::LatticeProviderError> for ProviderError {
  fn from(e: vino_provider_lattice::error::LatticeProviderError) -> Self {
    ProviderError::Lattice(e.to_string())
  }
}

impl From<vino_lattice::Error> for ProviderError {
  fn from(e: vino_lattice::Error) -> Self {
    ProviderError::Lattice(e.to_string())
  }
}

impl From<SchematicError> for ProviderError {
  fn from(e: SchematicError) -> Self {
    ProviderError::SchematicError(e.to_string())
  }
}
