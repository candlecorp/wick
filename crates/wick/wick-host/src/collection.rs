use std::collections::HashMap;
use std::sync::Arc;

use flow_component::{Component, ComponentError, RuntimeCallback};
use wick_interface_types::*;
use wick_packet::{Invocation, PacketStream};
use wick_rpc::RpcHandler;

use crate::ComponentHost;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
#[must_use]
pub struct HostComponent {
  host: Arc<ComponentHost>,
  signature: ComponentSignature,
}

impl HostComponent {
  pub fn new(host: ComponentHost) -> Self {
    let signature: ComponentSignature = host.get_signature().unwrap();

    Self {
      host: Arc::new(host),
      signature,
    }
  }
}

impl Component for HostComponent {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    _data: Option<flow_component::Value>,
    _callback: Arc<RuntimeCallback>,
  ) -> flow_component::BoxFuture<Result<PacketStream, ComponentError>> {
    let fut = self.host.invoke(invocation, stream);

    Box::pin(async move {
      let outputs = fut.await.map_err(ComponentError::new)?;

      Ok(outputs)
    })
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }
}

impl RpcHandler for HostComponent {}

#[cfg(test)]
mod tests {

  use anyhow::Result as TestResult;
  use flow_component::panic_callback;
  use tokio_stream::StreamExt;
  use wick_packet::{packet_stream, Entity, Packet};

  use super::*;
  use crate::ComponentHostBuilder;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let builder = ComponentHostBuilder::from_manifest_url("./manifests/logger.yaml", false, &[]).await?;
    let mut host = builder.build();
    host.start(Some(0)).await?;
    let collection = HostComponent::new(host);
    let input = "Hello world";

    let job_payload = packet_stream![("input", input)];

    let invocation = Invocation::new(Entity::test(file!()), Entity::local("logger"), None);
    let mut outputs = collection
      .handle(invocation, job_payload, None, panic_callback())
      .await?;
    let output = outputs.next().await.unwrap().unwrap();

    println!("output: {:?}", output);
    assert_eq!(output, Packet::encode("output", input));
    Ok(())
  }
}
