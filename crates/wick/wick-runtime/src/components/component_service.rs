use tracing::Instrument;

use crate::dev::prelude::*;
type Result<T> = std::result::Result<T, ComponentError>;

pub(crate) struct NativeComponentService {
  signature: ComponentSignature,
  component: SharedComponent,
}

impl NativeComponentService {
  pub(crate) fn new(component: SharedComponent) -> Self {
    Self {
      signature: component.signature().clone(),
      component,
    }
  }
}

impl Component for NativeComponentService {
  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }

  fn handle(
    &self,
    invocation: Invocation,
    config: Option<RuntimeConfig>,
    callback: LocalScope,
  ) -> flow_component::BoxFuture<std::result::Result<PacketStream, flow_component::ComponentError>> {
    let component = self.component.clone();

    let task = async move { component.handle(invocation, config, callback).await };
    Box::pin(task)
  }
}

impl InvocationHandler for NativeComponentService {
  fn get_signature(&self) -> Result<ComponentSignature> {
    Ok(self.signature.clone())
  }

  fn invoke(
    &self,
    invocation: Invocation,
    config: Option<RuntimeConfig>,
  ) -> Result<BoxFuture<Result<InvocationResponse>>> {
    let tx_id = invocation.tx_id();

    let span = info_span!(parent:invocation.span(),"runtime:handle");
    let fut = self.handle(invocation, config, Default::default());

    let task = async move {
      Ok(crate::dispatch::InvocationResponse::Stream {
        tx_id,
        rx: fut.instrument(span).await.map_err(ScopeError::NativeComponent)?,
      })
    };
    Ok(Box::pin(task))
  }
}
