use async_trait::async_trait;
use tracing::Instrument;
use uuid::Uuid;
use wick_packet::{Invocation, PacketStream};
use wick_rpc::error::RpcError;
use wick_rpc::{RpcHandler, RpcResult};

use crate::dev::prelude::*;

#[derive(Debug, Default)]
struct State {}

#[derive(Clone, Copy, Debug)]
pub struct Collection {
  network_id: Uuid,
}

impl Collection {
  #[must_use]
  pub fn new(network_id: Uuid) -> Self {
    Self { network_id }
  }
}

#[async_trait]
impl RpcHandler for Collection {
  async fn invoke(&self, invocation: Invocation, stream: PacketStream) -> RpcResult<PacketStream> {
    let target_url = invocation.target_url();

    let span = debug_span!(
      "invoke",
      network_id = %self.network_id,
      target =  %invocation.target
    );

    let network = NetworkService::for_id(&self.network_id)
      .ok_or_else(|| Box::new(RpcError::CollectionError(format!("Network '{}' not found", target_url))))?;

    trace!(target = %target_url, "invoking");

    let result: InvocationResponse = network
      .invoke(invocation, stream)
      .map_err(|e| RpcError::CollectionError(e.to_string()))?
      .instrument(span)
      .await
      .map_err(|e| RpcError::CollectionError(e.to_string()))?;

    match result.ok() {
      Ok(stream) => Ok(stream),
      Err(msg) => Err(Box::new(RpcError::CollectionError(format!(
        "Invocation failed: {}",
        msg
      )))),
    }
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let addr = NetworkService::for_id(&self.network_id).ok_or_else(|| {
      Box::new(RpcError::CollectionError(format!(
        "Network '{}' not found",
        self.network_id
      )))
    })?;
    let signature = addr
      .get_signature()
      .map_err(|e| RpcError::CollectionError(e.to_string()))?;
    Ok(vec![HostedType::Collection(signature)])
  }
}

#[cfg(test)]
mod tests {

  use wick_packet::{packet_stream, Entity, Packet};

  use super::*;
  use crate::test::prelude::{assert_eq, *};
  type Result<T> = anyhow::Result<T>;

  async fn request_log(collection: &Collection, data: &str) -> Result<String> {
    let stream = packet_stream!(("input", data));

    let invocation = Invocation::new(Entity::test(file!()), Entity::local("simple"), None);
    let outputs = collection.invoke(invocation, stream).await?;
    let mut packets: Vec<_> = outputs.collect().await;
    println!("packets: {:#?}", packets);
    let _ = packets.pop();
    let actual = packets.pop().unwrap().unwrap();

    println!("doc_id: {:?}", actual);
    assert_eq!(actual, Packet::encode("output", data));
    Ok(actual.payload.deserialize().unwrap())
  }

  #[test_logger::test(tokio::test)]
  async fn test_request_log() -> TestResult<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/v0/simple.wafl").await?;

    let collection = Collection::new(network_id);
    let user_data = "string to log";
    let result = request_log(&collection, user_data).await?;
    print!("Result: {}", result);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_list() -> TestResult<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/v0/simple.wafl").await?;
    let collection = Collection::new(network_id);
    let list = collection.get_list()?;
    println!("components on network : {:?}", list);
    assert_eq!(list.len(), 1);
    Ok(())
  }
}
