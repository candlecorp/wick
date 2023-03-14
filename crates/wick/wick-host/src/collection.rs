use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use wick_interface_types::*;
use wick_packet::{Invocation, PacketStream};
use wick_rpc::error::RpcError;
use wick_rpc::{RpcHandler, RpcResult};

use crate::Host;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Collection {
  host: Arc<Host>,
}

impl Collection {}

impl From<Host> for Collection {
  fn from(host: Host) -> Self {
    Self { host: Arc::new(host) }
  }
}

#[async_trait]
impl RpcHandler for Collection {
  async fn invoke(&self, invocation: Invocation, stream: PacketStream) -> RpcResult<PacketStream> {
    let outputs = self.host.invoke(invocation, stream).await.map_err(RpcError::boxed)?;

    Ok(outputs)
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let collection: ComponentSignature = self.host.get_signature().map_err(RpcError::boxed)?;

    Ok(vec![HostedType::Collection(collection)])
  }
}

#[cfg(test)]
mod tests {
  use std::convert::TryFrom;

  use anyhow::Result as TestResult;
  use tokio_stream::StreamExt;
  use wick_packet::{packet_stream, Entity, Packet};

  use super::*;
  use crate::HostBuilder;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let builder = HostBuilder::try_from("./manifests/logger.yaml")?;
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
