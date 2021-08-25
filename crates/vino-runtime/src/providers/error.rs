use thiserror::Error;

use crate::dev::prelude::*;

#[derive(Error, Debug)]
pub enum ProviderError {
  #[error(transparent)]
  InvocationError(#[from] InvocationError),
  #[error("Provider uninitialized")]
  Uninitialized,
  #[error(transparent)]
  WasmProviderError(#[from] vino_provider_wasm::Error),
  #[error("Failed to create a raw WebAssembly host")]
  WapcError,
  #[error(transparent)]
  ConversionError(#[from] ConversionError),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  ActixMailboxError(#[from] MailboxError),
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
  #[error("Lattice error: {0}")]
  Lattice(String),
}

impl From<vino_provider_lattice::error::LatticeProviderError> for ProviderError {
  fn from(e: vino_provider_lattice::error::LatticeProviderError) -> Self {
    ProviderError::Lattice(e.to_string())
  }
}
