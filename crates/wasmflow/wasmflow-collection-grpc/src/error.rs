use thiserror::Error;

#[derive(Error, Debug)]
pub enum GrpcError {
  #[error("Component error : {0}")]
  ComponentError(String),
  #[error(transparent)]
  RpcError(#[from] wasmflow_rpc::error::RpcClientError),
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

  #[error("Component '{0}' not found. Valid components are: {}", .1.join(", "))]
  ComponentNotFound(String, Vec<String>),
}
