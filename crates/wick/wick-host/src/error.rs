use thiserror::Error;
use wick_runtime::error::RuntimeError;

#[derive(Error, Debug)]
pub enum HostError {
  #[error("No runtime started yet")]
  NoRuntime,

  #[error("Runtime already started")]
  AlreadyRunning,

  #[error(transparent)]
  RuntimeError(#[from] Box<wick_runtime::Error>),

  #[error(transparent)]
  Resource(#[from] wick_runtime::resources::ResourceError),

  #[error("General error : {0}")]
  Other(String),
}

impl From<RuntimeError> for HostError {
  fn from(e: RuntimeError) -> Self {
    HostError::RuntimeError(Box::new(e))
  }
}
