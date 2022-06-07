use seeded_random::{Random, Seed};
use wasmflow_rpc::error::RpcError;
use wasmflow_rpc::{RpcHandler, RpcResult};
use wasmflow_sdk::sdk::stateful::NativeDispatcher;
use wasmflow_sdk::sdk::Invocation;
use wasmflow_sdk::types::HostedType;
use wasmflow_transport::TransportStream;

use crate::error::NativeError;

#[derive(Debug)]
pub struct Context {
  rng: Random,
}

impl Clone for Context {
  fn clone(&self) -> Self {
    Self {
      rng: Random::from_seed(self.rng.seed()),
    }
  }
}

impl Context {
  pub(crate) fn new(seed: Seed) -> Self {
    let rng = Random::from_seed(seed);
    Self { rng }
  }
}

#[derive(Clone, Debug)]
#[must_use]
pub struct Collection {
  context: Context,
}

impl From<NativeError> for Box<RpcError> {
  fn from(e: NativeError) -> Self {
    Box::new(RpcError::CollectionError(e.to_string()))
  }
}

impl Collection {
  pub fn new(seed: Seed) -> Self {
    let context = Context::new(seed);
    Self { context }
  }
}

#[async_trait::async_trait]
impl RpcHandler for Collection {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<TransportStream> {
    let context = self.context.clone();
    let dispatcher = crate::components::ComponentDispatcher::default();
    let result = dispatcher.dispatch(invocation, context).await;
    let stream = result.map_err(|e| RpcError::CollectionError(e.to_string()))?;

    Ok(TransportStream::from_packetstream(stream))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let signature = crate::components::get_signature();
    Ok(vec![HostedType::Collection(signature)])
  }
}

#[cfg(test)]
mod tests {

  use serde::de::DeserializeOwned;
  use tracing::debug;
  use wasmflow_entity::Entity;
  use wasmflow_packet::PacketMap;

  static SEED: u64 = 1000;

  use super::*;
  type Result<T> = std::result::Result<T, NativeError>;

  async fn invoke_one<T>(component: &str, payload: PacketMap, port: &str) -> Result<T>
  where
    T: DeserializeOwned,
  {
    let transport_map = payload;
    println!("TransportMap: {:?}", transport_map);
    let collection = Collection::new(Seed::unsafe_new(SEED));

    let entity = Entity::local(component);
    let invocation = Invocation::new_test(file!(), entity, transport_map, None);

    let mut outputs = collection.invoke(invocation).await.unwrap();
    let packets = outputs.drain_port(port).await?;
    let output = packets[0].clone();
    println!("Received payload from port '{}': {:?}", output.port, output.payload);
    Ok(output.payload.deserialize()?)
  }

  #[test_logger::test(tokio::test)]
  async fn test_log() -> Result<()> {
    let input = "some_input";
    let job_payload = crate::components::core::log::Inputs {
      input: input.to_owned(),
    };
    println!("Inputs: {:?}", job_payload);

    let payload: String = invoke_one("core::log", job_payload.into(), "output").await?;

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "some_input");

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn list() -> Result<()> {
    let collection = Collection::new(Seed::unsafe_new(SEED));
    let signature = crate::components::get_signature();
    let components = signature.components.inner();

    let response = collection.get_list().unwrap();

    debug!("list response : {:?}", response);

    assert_eq!(components.len(), 9);
    assert_eq!(response, vec![HostedType::Collection(signature)]);

    Ok(())
  }
}
