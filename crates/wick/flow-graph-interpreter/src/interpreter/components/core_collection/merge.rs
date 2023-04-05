use std::collections::HashMap;

use futures::StreamExt;
use serde_json::Value;
use tracing::Instrument;
use crate::BoxFuture;
use crate::{Component, ExecutionError};

#[derive(Default)]
pub(crate) struct MergeComponent {}

#[derive(serde::Deserialize)]
pub(crate) struct MergeConfig {
  inputs: wick_interface_types::FieldMap,
}

impl Component for MergeComponent {
  fn handle(
    &self,
    mut payload: wick_packet::StreamMap,
    data: Option<Value>,
  ) -> BoxFuture<Result<FrameStream, crate::BoxError>> {
    let task = async move {
      let data = data.ok_or(ExecutionError::InvalidMergeConfig)?;
      let data: MergeConfig = serde_json::from_value(data).map_err(|_| ExecutionError::InvalidMergeConfig)?;
      let mut map = HashMap::new();
      let mut streams = Vec::new();
      for field in data.inputs.inner().keys() {
        let stream = payload.take(field)?;
        streams.push(stream);
        while let next = stream.next().await {
          if next.is_done() {
            break;
          }
        }
        let payload: serde_value::Value = payload.take(field)?;
        trace!(input=%field,?payload,"merging");
        map.insert(field.clone(), payload);
      }
      loop {
        let futures = streams.iter_mut().map(|s| s.next());
        let results = futures::future::join_all(futures).await;
        let mut merged = HashMap::new();
        for (field, result) in data.inputs.inner().keys().zip(results) {
          if result.is_none() {

          }
          let payload: serde_value::Value = result?;
          trace!(input=%field,?payload,"merging");
          merged.insert(field.clone(), payload);
        }
        let final = results.into_iter().map(|r| r.map(|r|{}));
      }

      let messages = vec![
        TransportWrapper::new("output", MessageTransport::success(&map)),
        TransportWrapper::done("output"),
      ];

      let stream = FrameStream::new(tokio_stream::iter(messages.into_iter()));
      Ok(stream)
    };
    Box::pin(task)
  }
}
