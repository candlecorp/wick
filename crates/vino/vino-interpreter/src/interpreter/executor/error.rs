use uuid::Uuid;

use crate::interpreter::error::StateError;
use crate::BoxError;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ExecutionError {
  #[error("Error in internal channel")]
  ChannelError(crate::interpreter::channel::error::Error),
  #[error(transparent)]
  InvalidState(#[from] StateError),
  #[error("Payload does not contain message for port '{0}'")]
  MissingInput(String),
  #[error("Channel send failure")]
  ChannelSend,
  #[error("Transaction already running")]
  AlreadyStarted,
  #[error("Error executing provider {0}")]
  ProviderError(ProviderError),
  #[error("Transaction '{0}' missing")]
  MissingTx(Uuid),
  #[error("Channel stream has already been removed")]
  ChannelTaken,
}

#[derive(Debug)]
pub struct ProviderError(BoxError);
impl std::error::Error for ProviderError {}
impl std::fmt::Display for ProviderError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
impl PartialEq for ProviderError {
  fn eq(&self, other: &Self) -> bool {
    self.0.to_string() == other.0.to_string()
  }
}

impl From<BoxError> for ExecutionError {
  fn from(e: BoxError) -> Self {
    ExecutionError::ProviderError(ProviderError(e))
  }
}
