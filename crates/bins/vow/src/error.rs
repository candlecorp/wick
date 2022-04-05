use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VowError {
  #[error("{0}")]
  WasmProvider(String),
  #[error("Component panicked: {0}")]
  ComponentPanic(Box<vino_rpc::Error>),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error("JSON Serialization/Deserialization error : {0}")]
  JsonError(String),
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
  #[error(transparent)]
  CliError(#[from] vino_provider_cli::Error),
  #[error("{0}")]
  TestError(String),
  #[error("Invalid claims: {0}")]
  ClaimsError(String),
  #[error("General error: {0}")]
  Other(String),
  #[error("Read error for '{0}' : {1}")]
  NotFound(PathBuf, String),
  #[error("Internal Error: {0}")]
  InternalError(u32),
  #[error("Component '{0}' not found. Valid components are: {}", .1.join(", "))]
  ComponentNotFound(String, Vec<String>),

  #[error("{0}")]
  GeneralError(String),
}

impl From<serde_json::error::Error> for VowError {
  fn from(e: serde_json::error::Error) -> Self {
    VowError::JsonError(e.to_string())
  }
}

impl From<vino_test::Error> for VowError {
  fn from(e: vino_test::Error) -> Self {
    VowError::TestError(e.to_string())
  }
}

impl From<vino_provider_wasm::Error> for VowError {
  fn from(e: vino_provider_wasm::Error) -> Self {
    VowError::WasmProvider(e.to_string())
  }
}

impl From<String> for VowError {
  fn from(e: String) -> Self {
    Self::GeneralError(e)
  }
}
