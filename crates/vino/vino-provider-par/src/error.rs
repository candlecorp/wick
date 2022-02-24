use std::env::VarError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParError {
  #[error("Component error : {0}")]
  ComponentError(String),
  #[error(transparent)]
  RpcError(#[from] vino_rpc::error::RpcClientError),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error("JSON Serialization/Deserialization error : {0}")]
  JsonError(String),
  #[error("Could not extract claims from component. Is it a signed WebAssembly module?")]
  ClaimsExtraction,
  #[error("Error sending output to stream.")]
  SendError,
  #[error("Internal Error : {0}")]
  Other(String),
  #[error("Internal Error : {0}")]
  InternalError(u32),
  #[error("Could not create KeyPair from invalid seed")]
  KeyPairFailed,
  #[error("Component '{0}' not found. Valid components are: {}", .1.join(", "))]
  ComponentNotFound(String, Vec<String>),

  /// Provider Archive Error
  #[error(transparent)]
  Par(#[from] vino_par::Error),

  #[error(transparent)]
  EnvLookupFailed(#[from] shellexpand::LookupError<VarError>),
}

impl From<serde_json::error::Error> for ParError {
  fn from(e: serde_json::error::Error) -> Self {
    ParError::JsonError(e.to_string())
  }
}
