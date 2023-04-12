use flow_component::{ComponentError, Operation};
use serde_json::Value;
use wick_packet::{packet_stream, PacketStream};

use crate::interpreter::executor::error::ExecutionError;
use crate::BoxFuture;
#[derive(Default)]
pub(crate) struct SenderOperation {}

#[derive(serde::Deserialize)]
pub(crate) struct SenderData {
  output: Value,
}

impl Operation for SenderOperation {
  fn handle(
    &self,
    _payload: wick_packet::StreamMap,
    data: Option<Value>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let task = async move {
      let value = data.ok_or(ComponentError::new(ExecutionError::InvalidSenderData))?;
      let data: SenderData =
        serde_json::from_value(value).map_err(|_| ComponentError::new(ExecutionError::InvalidSenderData))?;
      Ok(packet_stream!(("output", data.output)))
    };
    Box::pin(task)
  }
}
