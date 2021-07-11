use rayon::ThreadPoolBuildError;
use thiserror::Error;
use vino_provider::ComponentSignature;

#[derive(Error, Debug)]
pub enum VowError {
  #[error(transparent)]
  LoggerError(#[from] logger::error::LoggerError),
  #[error(transparent)]
  ComponentError(#[from] vino_runtime::error::ComponentError),
  #[error(transparent)]
  ThreadPoolBuildError(#[from] ThreadPoolBuildError),
  #[error(transparent)]
  WapcError(#[from] wapc::errors::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error("JSON Serialization/Deserialization error : {0}")]
  JsonError(String),
  #[error(transparent)]
  TransportError(#[from] vino_transport::error::TransportError),
  #[error("Invalid claims : {0}")]
  ClaimsError(String),
  #[error("General error : {0}")]
  Other(String),
  #[error("Internal Error : {0}")]
  InternalError(u32),
  #[error("Component '{0}' not found. Valid components are: {}", .1.join(", "))]
  ComponentNotFound(String, Vec<String>),
}

impl From<serde_json::error::Error> for VowError {
  fn from(e: serde_json::error::Error) -> Self {
    VowError::JsonError(e.to_string())
  }
}
