use std::collections::HashMap;

use serde_json::Value;
use wasmflow_transport::{MessageTransport, TransportStream, TransportWrapper};

use crate::{Component, ExecutionError};

#[derive(Default)]
pub(crate) struct MergeComponent {}

#[derive(serde::Deserialize)]
pub(crate) struct MergeConfig {
  inputs: wasmflow_interface::FieldMap,
}

impl Component for MergeComponent {
  fn handle(
    &self,
    mut payload: wasmflow_transport::TransportMap,
    data: Option<Value>,
  ) -> futures::future::BoxFuture<Result<TransportStream, crate::BoxError>> {
    let task = async move {
      let data = data.ok_or(ExecutionError::InvalidMergeConfig)?;
      let data: MergeConfig = serde_json::from_value(data).map_err(|_| ExecutionError::InvalidMergeConfig)?;
      let mut map = HashMap::new();
      let span = trace_span!("aggregating inputs");
      let _guard = span.enter();
      for field in data.inputs.inner().keys() {
        let payload: serde_value::Value = payload.consume(field)?;
        trace!(input=%field,?payload,"merging");
        map.insert(field.clone(), payload);
      }
      let messages = vec![
        TransportWrapper::new("output", MessageTransport::success(&map)),
        TransportWrapper::done("output"),
      ];

      let stream = TransportStream::new(tokio_stream::iter(messages.into_iter()));
      Ok(stream)
    };
    Box::pin(task)
  }
}
