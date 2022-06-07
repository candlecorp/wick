use serde_json::Value;
use wasmflow_transport::{MessageTransport, TransportStream, TransportWrapper};

use crate::{Component, ExecutionError};

#[derive(Default)]
pub(crate) struct SenderComponent {}

#[derive(serde::Deserialize)]
pub(crate) struct SenderData {
  output: Value,
}

impl Component for SenderComponent {
  fn handle(
    &self,
    _payload: wasmflow_transport::TransportMap,
    data: Option<Value>,
  ) -> futures::future::BoxFuture<Result<TransportStream, crate::BoxError>> {
    let task = async move {
      let value = data.ok_or(ExecutionError::InvalidSenderData)?;
      let data: SenderData = serde_json::from_value(value).map_err(|_| ExecutionError::InvalidSenderData)?;
      let packet = MessageTransport::success(&data.output);
      let transport = TransportWrapper::new("output", packet);
      let messages = vec![transport, TransportWrapper::done("output")];
      let stream = TransportStream::new(tokio_stream::iter(messages.into_iter()));
      Ok(stream)
    };
    Box::pin(task)
  }
}
