use thiserror::Error;
use vino_rpc::error::RpcError;

#[derive(Error, Debug)]
pub enum WasmProviderError {
  #[error("Could not extract claims signature from WASM module : {0}")]
  ClaimsError(String),
  #[error("Could not validate claims : {0}")]
  ClaimsInvalid(String),
  #[error("Component error : {0}")]
  ComponentError(String),

  #[error("WASM provider requested data for a nonexistant call.")]
  TxNotFound,

  #[error(transparent)]
  WapcError(#[from] wapc::errors::Error),

  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error("Could not load reference: {0}")]
  Loading(String),
  #[error("JSON Serialization/Deserialization error : {0}")]
  JsonError(String),
  #[error(transparent)]
  TransportError(#[from] vino_transport::error::TransportError),

  #[error("WebAssembly engine failed: {0}")]
  EngineFailure(String),

  #[error("Could not extract claims from component. Is it a signed WebAssembly module?")]
  ClaimsExtraction,
  #[error("Error sending output to stream.")]
  SendError,
  #[error("{0}")]
  Other(String),
  #[error("Internal Error : {0}")]
  InternalError(u32),
  #[error("Could not create KeyPair from invalid seed")]
  KeyPairFailed,
  #[error("Component '{0}' not found. Valid components are: {}", .1.join(", "))]
  ComponentNotFound(String, Vec<String>),

  #[error(transparent)]
  Wasi(#[from] crate::wasi::error::WasiConfigError),
}

impl From<serde_json::error::Error> for WasmProviderError {
  fn from(e: serde_json::error::Error) -> Self {
    WasmProviderError::JsonError(e.to_string())
  }
}

impl From<WasmProviderError> for Box<RpcError> {
  fn from(e: WasmProviderError) -> Self {
    Box::new(RpcError::ProviderError(e.to_string()))
  }
}

impl From<vino_loader::Error> for WasmProviderError {
  fn from(e: vino_loader::Error) -> Self {
    WasmProviderError::Loading(e.to_string())
  }
}

#[derive(Error, Debug)]
pub enum LinkError {
  #[error("{0}")]
  EntityFailure(String),
  #[error("Component '{0}' can't call a link to itself.")]
  Circular(String),
  #[error("{0}")]
  CallFailure(String),
}

impl From<vino_entity::Error> for LinkError {
  fn from(e: vino_entity::Error) -> Self {
    LinkError::EntityFailure(e.to_string())
  }
}
