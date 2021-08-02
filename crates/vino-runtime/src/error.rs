use itertools::join;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use crate::dev::prelude::*;
use crate::schematic_service::handlers::transaction_update::TransactionUpdate;

#[derive(Error, Debug)]
pub enum SchematicError {
  #[error("Schematic model not initialized")]
  ModelNotInitialized,
  #[error("Transaction {0} not found")]
  TransactionNotFound(String),
  #[error("Instance {0} not found")]
  InstanceNotFound(String),
  #[error("Schematic failed pre-request condition")]
  FailedPreRequestCondition(String),
  #[error("Schematic channel closed while data still available. This can happen when acting on output before waiting for the system to receive the final close and may not be a problem.")]
  SchematicClosedEarly,
  #[error("Model invalid after validation: {0}")]
  InvalidModel(u32),
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error(transparent)]
  ValidationError(#[from] ValidationError),
  #[error(transparent)]
  ComponentError(#[from] ProviderError),
  #[error(transparent)]
  EntityError(#[from] vino_entity::Error),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error(transparent)]
  TransactionChannelError(#[from] SendError<TransactionUpdate>),
  #[error(transparent)]
  ModelError(#[from] SchematicModelError),
  #[error(transparent)]
  DefaultsError(#[from] serde_json::error::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  ManifestError(#[from] vino_manifest::Error),
}

#[derive(Error, Debug)]
pub enum NetworkError {
  #[error("Network not started")]
  NotStarted,
  #[error("Schematic {0} not found")]
  SchematicNotFound(String),
  #[error("Error initializing: {}", join(.0, ", "))]
  InitializationError(Vec<SchematicError>),
  #[error("Maximum number of tries reached when resolving internal schematic references")]
  MaxTriesReached,
  #[error(transparent)]
  SchematicError(#[from] SchematicError),
  #[error(transparent)]
  ComponentError(#[from] ProviderError),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error("Error executing request: {0}")]
  ExecutionError(String),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  RpcHandlerError(#[from] Box<vino_rpc::Error>),
}

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
  #[error(transparent)]
  RpcUpstreamError(#[from] tonic::Status),
  #[error(transparent)]
  OutputError(#[from] vino_component::error::DeserializationError),
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
}

#[derive(Error, Debug, Clone, Copy)]
pub struct ConversionError(pub &'static str);

impl std::fmt::Display for ConversionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.0)
  }
}

#[derive(Error, Debug, Clone, Copy)]
pub struct InternalError(pub i32);

impl std::fmt::Display for InternalError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("Internal Error: {}", self.0))
  }
}

impl From<i32> for InternalError {
  fn from(num: i32) -> Self {
    InternalError(num)
  }
}

#[derive(Error, Debug)]
pub enum CommonError {
  #[error("Provided KeyPair has no associated seed")]
  NoSeed,
  #[error("Failed to create KeyPair from seed")]
  KeyPairFailed,
  #[error(transparent)]
  DefaultsError(#[from] serde_json::error::Error),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
}

#[derive(Error, Debug)]
pub enum TransactionError {
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error("Upstream port {0} not found")]
  UpstreamNotFound(ConnectionTargetDefinition),
  #[error(transparent)]
  ManifestError(#[from] vino_manifest::Error),
}

#[derive(Error, Debug)]
#[error("Invocation error: {0}")]
pub struct InvocationError(pub String);

#[derive(Error, Debug)]
pub enum RuntimeError {
  #[error(transparent)]
  InvocationError(#[from] InvocationError),
  #[error(transparent)]
  InternalError(#[from] InternalError),
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error(transparent)]
  TransactionError(#[from] TransactionError),
  #[error(transparent)]
  ComponentError(#[from] ProviderError),
  #[error(transparent)]
  SchematicModelError(#[from] SchematicModelError),
  #[error(transparent)]
  NetworkError(#[from] NetworkError),
  #[error(transparent)]
  SchematicError(#[from] SchematicError),
  #[error(transparent)]
  TonicError(#[from] tonic::transport::Error),
  #[error(transparent)]
  RpcUpstreamError(#[from] tonic::Status),
  #[error(transparent)]
  EntityError(#[from] vino_entity::Error),
  #[error(transparent)]
  RpcError(#[from] vino_rpc::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  ManifestError(#[from] vino_manifest::Error),
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
  #[error(transparent)]
  OutputError(#[from] vino_component::error::DeserializationError),
  #[error(transparent)]
  ActixMailboxError(#[from] MailboxError),
  #[error(transparent)]
  RpcHandlerError(#[from] Box<vino_rpc::Error>),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
}
