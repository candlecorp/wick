use std::collections::HashMap;
use std::sync::Arc;

use wick_interface_types::*;
use wick_packet::{Invocation, PacketStream};
use wick_rpc::error::RpcError;
use wick_rpc::{BoxFuture, RpcHandler, RpcResult};

use crate::ComponentHost;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Collection {
  host: Arc<ComponentHost>,
}

impl Collection {}

impl From<ComponentHost> for Collection {
  fn from(host: ComponentHost) -> Self {
    Self { host: Arc::new(host) }
  }
}

impl RpcHandler for Collection {
  fn invoke(&self, invocation: Invocation, stream: PacketStream) -> BoxFuture<RpcResult<PacketStream>> {
    let fut = self.host.invoke(invocation, stream);

    Box::pin(async move {
      let outputs = fut.await.map_err(RpcError::boxed)?;

      Ok(outputs)
    })
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let collection: ComponentSignature = self.host.get_signature().map_err(RpcError::boxed)?;

    Ok(vec![HostedType::Component(collection)])
  }
}

#[cfg(test)]
mod tests {
  use std::convert::TryFrom;

  use anyhow::Result as TestResult;
  use tokio_stream::StreamExt;
  use wick_packet::{packet_stream, Entity, Packet};

  use super::*;
  use crate::ComponentHostBuilder;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let builder = ComponentHostBuilder::try_from("./manifests/logger.yaml")?;
    let mut host = builder.build();
    host.start(Some(0)).await?;
    let collection: Collection = host.into();
    let input = "Hello world";

    let job_payload = packet_stream![("input", input)];

    let invocation = Invocation::new(Entity::test(file!()), Entity::local("logger"), None);
    let mut outputs = collection.invoke(invocation, job_payload).await?;
    let output = outputs.next().await.unwrap().unwrap();

    println!("output: {:?}", output);
    assert_eq!(output, Packet::encode("output", input));
    Ok(())
  }
}
