use serde_json::Value;
use wick_packet::{packet_stream, PacketStream};

use crate::interpreter::executor::error::ExecutionError;
use crate::{BoxFuture, Operation};
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
  ) -> BoxFuture<Result<PacketStream, crate::BoxError>> {
    let task = async move {
      let value = data.ok_or(ExecutionError::InvalidSenderData)?;
      let data: SenderData = serde_json::from_value(value).map_err(|_| ExecutionError::InvalidSenderData)?;
      Ok(packet_stream!(("output", data.output)))
    };
    Box::pin(task)
  }
}
