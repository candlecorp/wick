use flow_graph_interpreter::Component;
use tracing::Instrument;
use wick_packet::{Invocation, PacketStream};
use wick_rpc::SharedRpcHandler;

use crate::dev::prelude::*;
use crate::BoxError;
type Result<T> = std::result::Result<T, ComponentError>;
use crate::BoxFuture;

pub(crate) struct NativeComponentService {
  signature: ComponentSignature,
  component: SharedRpcHandler,
}

impl NativeComponentService {
  pub(crate) fn new(component: SharedRpcHandler) -> Self {
    let HostedType::Component(signature) = &component.get_list().unwrap()[0];

    Self {
      component,
      signature: signature.clone(),
    }
  }
}

impl Component for NativeComponentService {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    _data: Option<serde_json::Value>,
  ) -> BoxFuture<std::result::Result<PacketStream, BoxError>> {
    let component = self.component.clone();

    let task = async move { Ok(component.invoke(invocation, stream).await?) };
    Box::pin(task)
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }

  fn shutdown(&self) -> BoxFuture<std::result::Result<(), BoxError>> {
    let component = self.component.clone();
    let task = async move {
      component.shutdown().await?;
      Ok(())
    };
    Box::pin(task)
  }
}

impl InvocationHandler for NativeComponentService {
  fn get_signature(&self) -> Result<ComponentSignature> {
    let component = self.component.clone();

    let mut list = component.get_list()?;
    drop(component);

    match list.swap_remove(0) {
      HostedType::Component(sig) => Ok(sig),
    }
  }

  fn invoke(&self, invocation: Invocation, stream: PacketStream) -> Result<BoxFuture<Result<InvocationResponse>>> {
    let tx_id = invocation.tx_id;
    let span = debug_span!("invoke", target =  %invocation.target);
    let fut = self.handle(invocation, stream, None);

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
