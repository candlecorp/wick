use futures::future::BoxFuture;
use serde_json::Value;
use wasmflow_sdk::v1::transport::{TransportMap, TransportStream, TransportWrapper};

use crate::{BoxError, Component};

#[derive(Default, Debug, Clone, Copy)]
pub struct OneShotComponent {}

impl Component for OneShotComponent {
  fn handle(&self, payload: TransportMap, _data: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    let mut messages = Vec::new();
    for wrapper in payload {
      let name = wrapper.port.clone();
      messages.push(wrapper);
      messages.push(TransportWrapper::done(name));
    }
    Box::pin(async move { Ok(TransportStream::new(tokio_stream::iter(messages.into_iter()))) })
  }
}
