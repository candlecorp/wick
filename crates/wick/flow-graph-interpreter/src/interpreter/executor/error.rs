use flow_component::ComponentError;
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

  #[error("Execution id:{0} hung and error_on_hung set")]
  HungTransaction(Uuid),

  #[error("{0}")]
  ComponentError(ComponentError),

  #[error("{0}")]
  OperationFailure(JoinError),
}

impl From<wasmrs_rx::Error> for ExecutionError {
  fn from(_e: wasmrs_rx::Error) -> Self {
    Self::ChannelSend
  }
}
