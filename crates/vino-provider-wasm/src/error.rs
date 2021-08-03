use thiserror::Error;
use vino_rpc::error::RpcError;

#[derive(Error, Debug)]
pub enum WasmProviderError {
  #[error(transparent)]
  LoggerError(#[from] logger::error::LoggerError),
  #[error("Component error : {0}")]
  ComponentError(String),
  #[error(transparent)]
  WapcError(#[from] wapc::errors::Error),
  #[error(transparent)]
  WascapError(#[from] vino_wascap::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  OciError(#[from] oci_utils::Error),
  #[error("JSON Serialization/Deserialization error : {0}")]
  JsonError(String),
  #[error(transparent)]
  TransportError(#[from] vino_transport::error::TransportError),
  #[error("Claims error: {0}")]
  ClaimsError(vino_wascap::wascap::Error),
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
