use flow_component::{Component, RuntimeCallback};
use tracing::Instrument;
use uuid::Uuid;
use wick_packet::{Invocation, PacketStream};
use wick_rpc::RpcHandler;

use crate::dev::prelude::*;

#[derive(Debug, Default)]
struct State {}

#[derive(Clone, Debug)]
pub struct EngineComponent {
  engine_id: Uuid,
  signature: ComponentSignature,
}

impl EngineComponent {
  #[must_use]
  pub fn new(engine_id: Uuid) -> Self {
    let addr = RuntimeService::for_id(&engine_id).unwrap();

    let signature = addr.get_signature().unwrap();

    Self { engine_id, signature }
  }
}

impl Component for EngineComponent {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    _data: Option<flow_component::Value>,
    _callback: std::sync::Arc<RuntimeCallback>,
  ) -> flow_component::BoxFuture<Result<PacketStream, flow_component::ComponentError>> {
    let target_url = invocation.target_url();

    let span = debug_span!(
      "invoke",
      engine_id = %self.engine_id,
      target =  %invocation.target
    );

    Box::pin(async move {
      let engine = RuntimeService::for_id(&self.engine_id)
        .ok_or_else(|| flow_component::ComponentError::message(&format!("Engine '{}' not found", target_url)))?;

      trace!(target = %target_url, "invoking");

      let result: InvocationResponse = engine
        .invoke(invocation, stream)
        .map_err(flow_component::ComponentError::new)?
        .instrument(span)
        .await
        .map_err(flow_component::ComponentError::new)?;

      match result.ok() {
        Ok(stream) => Ok(stream),
        Err(msg) => Err(flow_component::ComponentError::new(msg)),
      }
    })
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }
}

impl RpcHandler for EngineComponent {}

#[cfg(test)]
mod tests {

  use flow_component::panic_callback;
  use futures::StreamExt;
  use wick_packet::{packet_stream, Entity, Packet};

  use super::*;
  use crate::test::prelude::{assert_eq, *};
  type Result<T> = anyhow::Result<T>;

  async fn request_log(component: &EngineComponent, data: &str) -> Result<String> {
    let stream = packet_stream!(("MAIN_IN", data));

    let invocation = Invocation::new(Entity::test(file!()), Entity::local("simple"), None);
    let outputs = component.handle(invocation, stream, None, panic_callback()).await?;
    let mut packets: Vec<_> = outputs.collect().await;
    println!("packets: {:#?}", packets);
    let _ = packets.pop();
    let actual = packets.pop().unwrap().unwrap();

    println!("doc_id: {:?}", actual);
    assert_eq!(actual, Packet::encode("MAIN_OUT", data));
    Ok(actual.payload.deserialize().unwrap())
  }

  #[test_logger::test(tokio::test)]
  async fn test_request_log() -> Result<()> {
    let (_, engine_id) = init_engine_from_yaml("./manifests/v0/simple.yaml").await?;

    let component = EngineComponent::new(engine_id);
    let user_data = "string to log";
    let result = request_log(&component, user_data).await?;
    print!("Result: {}", result);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_list() -> Result<()> {
    let (_, engine_id) = init_engine_from_yaml("./manifests/v0/simple.yaml").await?;
    let component = EngineComponent::new(engine_id);
    let list = component.get_list()?;
    println!("components on engine : {:?}", list);
    assert_eq!(list.len(), 1);
    Ok(())
  }
}
