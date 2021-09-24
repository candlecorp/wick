use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use vino_lattice::lattice::Lattice;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  RpcHandler,
  RpcResult,
};

use crate::Error;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Provider {
  namespace: String,
  lattice: Arc<Lattice>,
}

impl Provider {
  pub async fn new(namespace: String, lattice: Arc<Lattice>) -> Result<Self, Error> {
    Ok(Self { namespace, lattice })
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    trace!("PROV:LATTICE:INVOKE:[{}]", entity);
    //
    let entity = Entity::Component(self.namespace.clone(), entity.name());

    let stream = self
      .lattice
      .invoke(entity, payload)
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;

    Ok(Box::pin(stream))
  }

  async fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let components = self
      .lattice
      .list_components(self.namespace.clone())
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;

    Ok(components)
  }
}

#[cfg(test)]
mod tests {

  use anyhow::Result as TestResult;
  use maplit::hashmap;
  use tokio_stream::StreamExt;
  use vino_lattice::lattice::LatticeBuilder;
  use vino_provider::native::prelude::*;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let lattice_builder = LatticeBuilder::new_from_env("test")?;
    let lattice = lattice_builder.build().await?;
    let ns = "some_namespace";

    lattice
      .handle_namespace(ns.to_owned(), || {
        Box::new(test_vino_provider::Provider::default())
      })
      .await?;

    let provider = Provider::new(ns.to_owned(), Arc::new(lattice)).await?;
    let user_data = "Hello world";

    let job_payload = TransportMap::with_map(hashmap! {
      "input".to_owned() => MessageTransport::messagepack(user_data),
    });

    let mut outputs = provider
      .invoke(Entity::component(ns, "test-component"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output: String = output.payload.try_into()?;

    println!("output: {:?}", output);
    assert_eq!(output, format!("TEST: {}", user_data));
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_error() -> TestResult<()> {
    let lattice_builder = LatticeBuilder::new_from_env("test")?;
    let lattice = lattice_builder.build().await?;
    let ns = "some_namespace";

    lattice
      .handle_namespace(ns.to_owned(), || {
        Box::new(test_vino_provider::Provider::default())
      })
      .await?;

    let provider = Provider::new(ns.to_owned(), Arc::new(lattice)).await?;
    let user_data = "Hello world";

    let job_payload = TransportMap::with_map(hashmap! {
      "input".to_owned() => MessageTransport::messagepack(user_data),
    });

    let mut outputs = provider
      .invoke(Entity::component(ns, "error"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    assert_eq!(
      output.payload,
      MessageTransport::error("This always errors".to_owned())
    );
    Ok(())
  }
}
