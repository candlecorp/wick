use std::collections::HashMap;
use std::sync::Arc;

use flow_component::{Component, ComponentError, LocalScope};
use wick_interface_types::*;
use wick_packet::{Invocation, PacketStream, RuntimeConfig};

use crate::{ComponentHost, Host};

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Context {
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
#[must_use]
pub struct HostComponent {
  id: String,
  host: Arc<ComponentHost>,
  signature: ComponentSignature,
}

impl HostComponent {
  pub fn new(host: ComponentHost) -> Self {
    let signature: ComponentSignature = host.get_signature(None, None).unwrap();

    Self {
      id: host.get_host_id().to_owned(),
      host: Arc::new(host),
      signature,
    }
  }
}

impl HostComponent {
  /// Returns the host id
  #[must_use]
  pub fn id(&self) -> &str {
    &self.id
  }
}

impl Component for HostComponent {
  fn handle(
    &self,
    invocation: Invocation,
    data: Option<RuntimeConfig>,
    _callback: LocalScope,
  ) -> flow_component::BoxFuture<Result<PacketStream, ComponentError>> {
    let fut = self.host.invoke(invocation, data);

    Box::pin(async move {
      let outputs = fut.await.map_err(ComponentError::new)?;

      Ok(outputs)
    })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }
}

#[cfg(test)]
mod tests {

  use anyhow::Result as TestResult;
  use tokio_stream::StreamExt;
  use wick_config::WickConfiguration;
  use wick_packet::{packet_stream, Entity, Packet};

  use super::*;
  use crate::ComponentHostBuilder;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let manifest = WickConfiguration::fetch("./manifests/logger.yaml", Default::default())
      .await?
      .finish()?
      .try_component_config()?;
    let mut builder = ComponentHostBuilder::default();
    builder.manifest(manifest);

    let mut host = builder.build()?;
    host.start(Some(0)).await?;
    let collection = HostComponent::new(host);
    let input = "Hello world";

    let job_payload = packet_stream![("input", input)];

    let invocation = Invocation::test(file!(), Entity::local("logger"), job_payload, None)?;
    let mut outputs = collection
      .handle(invocation, Default::default(), Default::default())
      .await?;
    let output = outputs.next().await.unwrap().unwrap();

    println!("output: {:?}", output);
    assert_eq!(output, Packet::encode("output", input));
    Ok(())
  }
}
