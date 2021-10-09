use thiserror::Error;

use crate::dev::prelude::*;
pub use crate::network_service::error::NetworkError;
pub use crate::providers::error::ProviderError;
pub use crate::schematic_service::error::SchematicError;

#[derive(Error, Debug, Clone, Copy)]
pub struct ConversionError(pub &'static str);

impl std::fmt::Display for ConversionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.0)
  }
}

#[derive(Error, Debug, Clone, Copy)]
pub enum InternalError {
  // Sync error
  E2001,
  // Network errors
  E5001,
  E5002, // Keypair
  E5101,
  E5102,
  E5103,
  E5004,
  // Schematic errors
  E6012,
  E6013,
  E6002,
  E6003,
  E6007,
  E6009,
  // Provider errors
  E7001,
  E7002,
  E7003,
  E7004,
  E7005,
  // Transaction errors
  E9001,
  E9002,
  E9004,
  E9005,
}

impl std::fmt::Display for InternalError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("Internal Error: {}", self))
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
  #[error("Loading/fetching failed: {0}")]
  Loading(String),
  #[error("Failed to read manifest: {0}")]
  Manifest(String),
  #[error("Uninitialized")]
  Uninitialized,
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
  #[error("{0}")]
  Lattice(String),
  #[error("{0}")]
  Serialization(String),
  #[error("{0}")]
  InitializationFailed(String),
}

impl From<vino_lattice::Error> for RuntimeError {
  fn from(e: vino_lattice::Error) -> Self {
    RuntimeError::Lattice(e.to_string())
  }
}

impl From<MailboxError> for RuntimeError {
  fn from(_: MailboxError) -> Self {
    RuntimeError::MailboxClosed
  }
}
