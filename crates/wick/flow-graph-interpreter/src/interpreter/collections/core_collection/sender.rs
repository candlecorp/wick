use serde_json::Value;
use wasmrs_rx::{FluxChannel, Observer};
use wick_packet::{Packet, PacketStream};

use crate::interpreter::executor::error::ExecutionError;
use crate::Operation;

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
  ) -> futures::future::BoxFuture<Result<PacketStream, crate::BoxError>> {
    let task = async move {
      let value = data.ok_or(ExecutionError::InvalidSenderData)?;
      let data: SenderData = serde_json::from_value(value).map_err(|_| ExecutionError::InvalidSenderData)?;
      let outstream = FluxChannel::new();

      outstream.send(Packet::encode("output", data.output))?;
      outstream.send(Packet::done("output"))?;

      Ok(PacketStream::new(Box::new(outstream)))
    };
    Box::pin(task)
  }
}
