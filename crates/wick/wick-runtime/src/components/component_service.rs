use flow_component::panic_callback;
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
    callback: Arc<RuntimeCallback>,
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
    let tx_id = invocation.tx_id;

    let span = debug_span!(parent:&invocation.span,"runtime:handle");
    let fut = self.handle(invocation, config, panic_callback());

    let task = async move {
      Ok(crate::dispatch::InvocationResponse::Stream {
        tx_id,
        rx: fut.instrument(span).await.map_err(EngineError::NativeComponent)?,
      })
    };
    Ok(Box::pin(task))
  }
}

#[cfg(test)]
mod test {

  // use std::sync::Arc;

  // use anyhow::Result;
  // use seeded_random::Seed;

  // use super::*;
  // use crate::test::prelude::assert_eq;

  // #[test_logger::test(tokio::test)]
  // async fn test_collection_component() -> Result<()> {
  //   let seed: u64 = 100000;
  //   let collection = NativeCollectionService::new(Arc::new(wick_stdlib::Collection::new(Seed::unsafe_new(seed))));

  //   let user_data = "This is my payload";

  //   let payload = vec![("input", user_data)].into();
  //   let invocation = Invocation::new(Entity::test("test"), Entity::local("core::log"), payload, None);
  //   let response = collection.invoke(invocation)?.await?;

  //   let mut rx = response.ok()?;
  //   let packets: Vec<_> = rx.collect().await;
  //   let p = packets.pop().unwrap().unwrap();
  //   assert_eq!(p, Packet::encode("output", user_data));

  //   Ok(())
  // }
}
