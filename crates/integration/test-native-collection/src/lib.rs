use async_trait::async_trait;
use wasmflow_rpc::error::RpcError;
use wasmflow_rpc::{RpcHandler, RpcResult};
use wasmflow_sdk::sdk::stateful::NativeDispatcher;
use wasmflow_sdk::sdk::Invocation;
use wasmflow_sdk::types::HostedType;
use wasmflow_transport::TransportStream;

use self::components::ComponentDispatcher;
pub mod components;

#[macro_use]
extern crate tracing;

#[derive(Clone, Debug)]
pub struct Context {}

#[derive(Clone)]
pub struct Collection {
  context: Context,
}

impl Collection {
  pub fn default() -> Self {
    Self { context: Context {} }
  }
}

#[async_trait]
impl RpcHandler for Collection {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<TransportStream> {
    let target = invocation.target_url();
    trace!("test collection invoke: {}", target);
    let context = self.context.clone();
    let dispatcher = ComponentDispatcher::default();
    let result = dispatcher
      .dispatch(invocation, context)
      .await
      .map_err(|e| RpcError::CollectionError(e.to_string()));
    trace!("test collection result: {}", target);
    let stream = result?;

    Ok(TransportStream::from_packetstream(stream))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    trace!("test collection get list");
    let signature = components::get_signature();
    Ok(vec![HostedType::Collection(signature)])
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use futures::prelude::*;
  use pretty_assertions::assert_eq;
  use tracing::*;
  use wasmflow_entity::Entity;
  use wasmflow_sdk::types::*;

  use super::*;
  use crate::components::test_component;

  #[test_logger::test(tokio::test)]
  async fn request() -> anyhow::Result<()> {
    let collection = Collection::default();
    let input = "some_input";
    let job_payload = test_component::Inputs {
      input: input.to_owned(),
    };

    let entity = Entity::local("test-component");
    let invocation = Invocation::new_test(file!(), entity, job_payload, None);

    let mut outputs = collection.invoke(invocation).await?;
    let packets: Vec<_> = outputs.drain_port("output").await?;
    let output = packets[0].clone();

    println!("Received payload from [{}]", output.port);
    let payload: String = output.payload.deserialize().unwrap();

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "TEST: some_input");

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn list() -> anyhow::Result<()> {
    let collection = Collection::default();

    let response = collection.get_list()?;

    debug!("list response : {:?}", response);

    assert_eq!(response.len(), 1);
    let expected = CollectionSignature {
      format: 1,
      version: "0.1.0".to_owned(),
      wellknown: vec![],
      name: Some("test-native-collection".to_owned()),
      components: HashMap::from([
        (
          "error".to_owned(),
          ComponentSignature {
            name: "error".to_string(),
            inputs: HashMap::from([("input".to_owned(), TypeSignature::String)]).into(),
            outputs: HashMap::from([("output".to_owned(), TypeSignature::String)]).into(),
          },
        ),
        (
          "test-component".to_owned(),
          ComponentSignature {
            name: "test-component".to_string(),
            inputs: HashMap::from([("input".to_owned(), TypeSignature::String)]).into(),
            outputs: HashMap::from([("output".to_owned(), TypeSignature::String)]).into(),
          },
        ),
      ])
      .into(),
      types: TypeMap::new(),
      config: TypeMap::new(),
    };
    assert_eq!(response[0], HostedType::Collection(expected));
    Ok(())
  }
}
