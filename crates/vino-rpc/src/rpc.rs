use serde::{
  Deserialize,
  Serialize,
};
use vino_runtime::{
  Invocation,
  MessagePayload,
  PortEntity,
};

use crate::{
  Error,
  Result,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum VinoRpcMessage {
  Invoke(Invocation),
  Output(OutputMessage),
  Close(CloseMessage),
  Error(String),
  Ping(String),
  Pong(String),
  Shutdown,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OutputMessage {
  pub entity: PortEntity,
  pub tx_id: String,
  pub payload: MessagePayload,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CloseMessage {
  pub entity: PortEntity,
  pub tx_id: String,
}

pub const OP_INVOKE: &str = "invoke";
pub const OP_OUTPUT: &str = "output";
pub const OP_CLOSE: &str = "close";
pub const OP_ERROR: &str = "error";
pub const OP_PING: &str = "ping";
pub const OP_PONG: &str = "pong";
pub const OP_SHUTDOWN: &str = "shutdown";

impl VinoRpcMessage {
  pub fn op_name(&self) -> &str {
    match self {
      VinoRpcMessage::Invoke(_) => OP_INVOKE,
      VinoRpcMessage::Output { .. } => OP_OUTPUT,
      VinoRpcMessage::Close { .. } => OP_CLOSE,
      VinoRpcMessage::Error(_) => OP_ERROR,
      VinoRpcMessage::Ping(_) => OP_PING,
      VinoRpcMessage::Pong(_) => OP_PONG,
      VinoRpcMessage::Shutdown => OP_SHUTDOWN,
    }
  }
  pub fn into_invocation(self) -> Result<Invocation> {
    match self {
      VinoRpcMessage::Invoke(i) => Ok(i),
      _ => Err(Error::ConversionError),
    }
  }
  pub fn into_output(self) -> Result<OutputMessage> {
    match self {
      VinoRpcMessage::Output(i) => Ok(i),
      _ => Err(Error::ConversionError),
    }
  }
  pub fn into_close(self) -> Result<CloseMessage> {
    match self {
      VinoRpcMessage::Close(i) => Ok(i),
      _ => Err(Error::ConversionError),
    }
  }
  pub fn into_error(self) -> Result<String> {
    match self {
      VinoRpcMessage::Error(i) => Ok(i),
      _ => Err(Error::ConversionError),
    }
  }
  pub fn into_ping(self) -> Result<String> {
    match self {
      VinoRpcMessage::Ping(i) => Ok(i),
      _ => Err(Error::ConversionError),
    }
  }
  pub fn into_pong(self) -> Result<String> {
    match self {
      VinoRpcMessage::Pong(i) => Ok(i),
      _ => Err(Error::ConversionError),
    }
  }
  pub fn into_shutdown(self) -> Result<()> {
    match self {
      VinoRpcMessage::Shutdown => Ok(()),
      _ => Err(Error::ConversionError),
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test_env_log::test(tokio::test)]
  async fn enforce_names() {
    assert_eq!(
      VinoRpcMessage::Invoke(Invocation::default()).op_name(),
      OP_INVOKE
    );
    assert_eq!(
      VinoRpcMessage::Output(OutputMessage::default()).op_name(),
      OP_OUTPUT
    );
    assert_eq!(
      VinoRpcMessage::Close(CloseMessage::default()).op_name(),
      OP_CLOSE
    );
    assert_eq!(VinoRpcMessage::Error(String::default()).op_name(), OP_ERROR);
    assert_eq!(VinoRpcMessage::Ping(String::default()).op_name(), OP_PING);
    assert_eq!(VinoRpcMessage::Pong(String::default()).op_name(), OP_PONG);
    assert_eq!(VinoRpcMessage::Shutdown.op_name(), OP_SHUTDOWN);
  }
}
