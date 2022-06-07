use async_trait::async_trait;
use tracing::Instrument;
use uuid::Uuid;
use wasmflow_rpc::error::RpcError;
use wasmflow_rpc::{RpcHandler, RpcResult};

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
  async fn invoke(&self, invocation: Invocation) -> RpcResult<TransportStream> {
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
      .invoke(invocation)
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

  use wasmflow_packet::PacketMap;

  use super::*;
  use crate::test::prelude::{assert_eq, *};
  type Result<T> = anyhow::Result<T>;

  async fn request_log(collection: &Collection, data: &str) -> Result<String> {
    let job_payload = PacketMap::from([("input", data)]);

    let invocation = Invocation::new_test(file!(), Entity::local("simple"), job_payload, None);
    let mut outputs = collection.invoke(invocation).await?;
    let output = outputs.drain_port("output").await?[0].clone();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output_data: String = output.payload.deserialize()?;

    println!("doc_id: {:?}", output_data);
    assert_eq!(output_data, data);
    Ok(output_data)
  }

  #[test_logger::test(tokio::test)]
  async fn test_request_log() -> TestResult<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/v0/simple.yaml").await?;

    let collection = Collection::new(network_id);
    let user_data = "string to log";
    let result = request_log(&collection, user_data).await?;
    print!("Result: {}", result);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_list() -> TestResult<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/v0/simple.yaml").await?;
    let collection = Collection::new(network_id);
    let list = collection.get_list()?;
    println!("components on network : {:?}", list);
    assert_eq!(list.len(), 1);
    Ok(())
  }
}
