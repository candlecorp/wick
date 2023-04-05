use tokio::task::JoinError;
use uuid::Uuid;

use crate::interpreter::error::StateError;

#[derive(thiserror::Error, Debug)]
pub enum ExecutionError {
  #[error("Error in internal channel: {0}")]
  ChannelError(crate::interpreter::channel::error::Error),

  #[error(transparent)]
  InvalidState(#[from] StateError),
  #[error("Payload does not contain message for port '{0}'")]
  MissingInput(String),
  #[error("Channel send failure")]
  ChannelSend,
  #[error("Transaction '{0}' hung and error_on_hung set")]
  HungTransaction(Uuid),
  #[error("Transaction '{0}' missing")]
  MissingTx(Uuid),
  #[error("{0}")]
  ComponentError(Box<dyn std::error::Error + Send + Sync>),

  #[error("{0}")]
  OperationFailure(JoinError),

  #[error("Sender configuration did not include valid data")]
  InvalidSenderData,

  #[error("Configuration for dynamic merge component invalid")]
  InvalidMergeConfig,
}

impl From<wasmrs_rx::Error> for ExecutionError {
  fn from(_e: wasmrs_rx::Error) -> Self {
    Self::ChannelSend
  }
}
