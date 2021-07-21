use thiserror::Error;

#[derive(Error, Debug)]
pub enum NativeError {
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  ComponentError(#[from] vino_component::Error),
  #[error(transparent)]
  JoinError(#[from] tokio::task::JoinError),
  #[error("Can not handle entity type {0}")]
  InvalidEntity(String),
  #[error("General error : {0}")]
  Other(String),
}
