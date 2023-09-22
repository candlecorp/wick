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
  TriggerError(#[from] Box<wick_trigger::Error>),

  #[error(transparent)]
  Resource(#[from] wick_trigger::resources::ResourceError),

  #[error("General error : {0}")]
  Other(String),
}

impl From<RuntimeError> for HostError {
  fn from(e: RuntimeError) -> Self {
    HostError::RuntimeError(Box::new(e))
  }
}

impl From<wick_trigger::Error> for HostError {
  fn from(e: wick_trigger::Error) -> Self {
    HostError::TriggerError(Box::new(e))
  }
}
