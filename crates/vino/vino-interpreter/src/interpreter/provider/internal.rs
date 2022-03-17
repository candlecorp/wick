use futures::future::BoxFuture;
use vino_schematic_graph::{SCHEMATIC_INPUT, SCHEMATIC_OUTPUT};
use vino_transport::{TransportMap, TransportStream, TransportWrapper};
use vino_types::ComponentMap;

use crate::{BoxError, Provider};

pub(crate) const INTERNAL_PROVIDER: &str = "__interpreter";

#[derive(Debug, Default)]
pub(crate) struct InternalProvider {
  signature: ComponentMap,
}

impl Provider for InternalProvider {
  fn handle(&self, operation: &str, payload: TransportMap) -> BoxFuture<Result<TransportStream, BoxError>> {
    trace!("INTERNAL:{}", operation);
    let is_passthrough = operation == SCHEMATIC_INPUT || operation == SCHEMATIC_OUTPUT;
    let stream = if is_passthrough {
      let mut messages = Vec::new();
      for wrapper in payload {
        let name = wrapper.port.clone();
        messages.push(wrapper);
        messages.push(TransportWrapper::done(name));
      }
      TransportStream::new(tokio_stream::iter(messages.into_iter()))
    } else {
      panic!("Internal component {} not handled.", operation);
    };
    Box::pin(async move { Ok(TransportStream::new(stream)) })
  }

  fn list(&self) -> &ComponentMap {
    &self.signature
  }
}
