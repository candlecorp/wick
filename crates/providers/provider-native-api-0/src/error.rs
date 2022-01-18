use thiserror::Error;

#[derive(Error, Debug)]
pub enum NativeError {
  #[error(transparent)]
  TransportError(#[from] vino_provider::native::prelude::TransportError),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  JoinError(#[from] tokio::task::JoinError),
  #[error("{0}")]
  Other(String),
}

impl From<&str> for NativeError {
  fn from(v: &str) -> Self {
    NativeError::Other(v.to_owned())
  }
}
