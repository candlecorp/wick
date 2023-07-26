use thiserror::Error;
use wick_rpc::error::RpcError;

#[derive(Error, Debug)]
pub enum WasmComponentError {
  #[error("Could not extract claims signature from WASM module : {0}")]
  ClaimsError(String),

  #[error("Could not validate claims : {0}")]
  ClaimsInvalid(String),

  #[error(transparent)]
  WasmRS(#[from] wasmrs::Error),

  #[error("Setup failed: {}", .0.msg)]
  Setup(wasmrs::PayloadError),

  #[error(transparent)]
  SetupSignature(wick_packet::Error),

  #[error("Setup failed, operation timed out.")]
  SetupTimeout,

  #[error(transparent)]
  IoError(#[from] std::io::Error),

  #[error(transparent)]
  Asset(#[from] wick_config::AssetError),

  #[error("JSON Serialization/Deserialization error : {0}")]
  JsonError(String),

  #[error("WebAssembly engine failed: {0}")]
  EngineFailure(String),

  #[error(transparent)]
  ContextInit(wasmrs_host::errors::Error),

  #[error("Could not extract claims from component. Is it a signed WebAssembly module?")]
  ClaimsExtraction,

  #[error("Operation '{0}' not found. Valid operations are: {}", .1.join(", "))]
  OperationNotFound(String, Vec<String>),

  #[error("Operation '__setup' not exported by the wasm module.")]
  SetupOperation,
}

impl From<serde_json::error::Error> for WasmComponentError {
  fn from(e: serde_json::error::Error) -> Self {
    WasmComponentError::JsonError(e.to_string())
  }
}

impl From<WasmComponentError> for Box<RpcError> {
  fn from(e: WasmComponentError) -> Self {
    Box::new(RpcError::Component(e.to_string()))
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
