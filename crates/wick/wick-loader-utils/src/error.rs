use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
  #[error("Component error : {0}")]
  ComponentError(String),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  OciError(#[from] wick_oci_utils::Error),
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
  #[error("Component '{0}' not found. Valid components are: {}", .1.join(", "))]
  ComponentNotFound(String, Vec<String>),
}
