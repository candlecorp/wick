use thiserror::Error;

type BoxedError = Box<dyn std::error::Error + Sync + std::marker::Send>;

#[derive(Error, Debug)]
pub enum RpcError {
  #[error("Deserialization error {0}")]
  RpcMessageError(&'static str),
  #[error("Message conversion error")]
  ConversionError,
  #[error("Error {0}")]
  Other(String),
  #[error(transparent)]
  VinoError(#[from] vino_runtime::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  UpstreamError(#[from] BoxedError),
  #[error(transparent)]
  JoinError(#[from] tokio::task::JoinError),
}

impl From<&str> for RpcError {
  fn from(s: &str) -> Self {
    Self::Other(s.to_string())
  }
}
