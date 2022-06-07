use thiserror::Error;
use wasmflow_rpc::error::RpcError;

#[derive(Error, Debug)]
pub enum WasmCollectionError {
  #[error("Could not extract claims signature from WASM module : {0}")]
  ClaimsError(String),
  #[error("Could not validate claims : {0}")]
  ClaimsInvalid(String),
  #[error("Component error : {0}")]
  ComponentError(String),

  #[error("WASM collection requested data for a nonexistent call.")]
  TxNotFound,

  #[error(transparent)]
  WapcError(#[from] wapc::errors::Error),

  #[error(transparent)]
  CodecError(#[from] wasmflow_codec::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error("Could not load reference: {0}")]
  Loading(String),
  #[error("JSON Serialization/Deserialization error : {0}")]
  JsonError(String),
  #[error(transparent)]
  TransportError(#[from] wasmflow_transport::error::TransportError),

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

impl From<serde_json::error::Error> for WasmCollectionError {
  fn from(e: serde_json::error::Error) -> Self {
    WasmCollectionError::JsonError(e.to_string())
  }
}

impl From<WasmCollectionError> for Box<RpcError> {
  fn from(e: WasmCollectionError) -> Self {
    Box::new(RpcError::CollectionError(e.to_string()))
  }
}

impl From<wasmflow_loader::Error> for WasmCollectionError {
  fn from(e: wasmflow_loader::Error) -> Self {
    WasmCollectionError::Loading(e.to_string())
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

impl From<wasmflow_entity::Error> for LinkError {
  fn from(e: wasmflow_entity::Error) -> Self {
    LinkError::EntityFailure(e.to_string())
  }
}
