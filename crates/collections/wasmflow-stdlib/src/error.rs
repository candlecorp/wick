use thiserror::Error;

#[derive(Error, Debug)]
pub enum NativeError {
  #[error(transparent)]
  Component(#[from] wasmflow_sdk::v1::error::Error),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  JoinError(#[from] tokio::task::JoinError),
  #[error("{0}")]
  Other(String),
  #[error("{0}")]
  Upstream(Box<dyn std::error::Error + Send + Sync>),
}

impl From<&str> for NativeError {
  fn from(v: &str) -> Self {
    NativeError::Other(v.to_owned())
  }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for NativeError {
  fn from(v: Box<dyn std::error::Error + Send + Sync>) -> Self {
    NativeError::Upstream(v)
  }
}
