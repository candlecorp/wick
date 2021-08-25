use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use crate::dev::prelude::*;
pub use crate::network_service::error::NetworkError;
pub use crate::providers::error::ProviderError;

#[derive(Error, Debug)]
pub enum SchematicError {
  #[error("Schematic model not initialized")]
  ModelNotInitialized,
  #[error("Transaction {0} not found")]
  TransactionNotFound(String),
  #[error("Instance {0} not found")]
  InstanceNotFound(String),
  #[error("Schematic failed pre-request condition: {0}")]
  FailedPreRequestCondition(String),
  #[error("Schematic channel closed while data still available. This can happen when the client disconnects early either due to an error or acting on the stream without waiting for it to complete.")]
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
  OutputError(#[from] vino_packet::error::DeserializationError),
  #[error("Mailbox closed")]
  MailboxClosed,
  #[error(transparent)]
  RpcHandlerError(#[from] Box<vino_rpc::Error>),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
}

impl From<MailboxError> for RuntimeError {
  fn from(_: MailboxError) -> Self {
    RuntimeError::MailboxClosed
  }
}
