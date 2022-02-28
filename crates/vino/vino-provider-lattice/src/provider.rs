use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use futures::executor::block_on;
use vino_lattice::Lattice;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcHandler, RpcResult};
use vino_transport::Invocation;

use crate::Error;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Provider {
  lattice_id: String,
  lattice: Arc<Lattice>,
}

impl Provider {
  pub async fn new(lattice_id: String, lattice: Arc<Lattice>) -> Result<Self, Error> {
    Ok(Self { lattice_id, lattice })
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<BoxedTransportStream> {
    let target_url = invocation.target_url();
    trace!("PROV:LATTICE:INVOKE:[{}]", target_url);

    let start = Instant::now();
    let stream = self
      .lattice
      .invoke(&self.lattice_id, invocation)
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;
    trace!(
      "PROV:LATTICE:INVOKE:[{}]:DURATION[{} ms]",
      target_url,
      start.elapsed().as_millis()
    );

    Ok(Box::pin(stream))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let components = block_on(self.lattice.list_components(self.lattice_id.clone()))
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;

    Ok(components)
  }
}

#[cfg(test)]
mod tests {

  use anyhow::Result as TestResult;
  use tokio_stream::StreamExt;
  use vino_lattice::LatticeBuilder;
  use vino_provider::native::prelude::*;
  use vino_rpc::SharedRpcHandler;

  use super::*;

  fn get_provider() -> SharedRpcHandler {
    Arc::new(test_vino_provider::Provider::default())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_component() -> TestResult<()> {
    let lattice_builder = LatticeBuilder::new_from_env("test")?;
    let lattice = lattice_builder.build().await?;
    let ns = "some_namespace";

    lattice.handle_namespace(ns.to_owned(), get_provider()).await?;

    let provider = Provider::new(ns.to_owned(), Arc::new(lattice)).await?;
    let user_data = "Hello world";

    let job_payload = TransportMap::from_map(HashMap::from([(
      "input".to_owned(),
      MessageTransport::messagepack(user_data),
    )]));
    let invocation = Invocation::new_test(file!(), Entity::component(ns, "test-component"), job_payload);

    let mut outputs = provider.invoke(invocation).await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output: String = output.payload.try_into()?;

    println!("output: {:?}", output);
    assert_eq!(output, format!("TEST: {}", user_data));
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_error() -> TestResult<()> {
    let lattice_builder = LatticeBuilder::new_from_env("test")?;
    let lattice = lattice_builder.build().await?;
    let ns = "some_namespace";

    lattice.handle_namespace(ns.to_owned(), get_provider()).await?;

    let provider = Provider::new(ns.to_owned(), Arc::new(lattice)).await?;
    let user_data = "Hello world";

    let job_payload = TransportMap::from_map(HashMap::from([(
      "input".to_owned(),
      MessageTransport::messagepack(user_data),
    )]));
    let invocation = Invocation::new_test(file!(), Entity::component(ns, "error"), job_payload);

    let mut outputs = provider.invoke(invocation).await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    assert_eq!(output.payload, MessageTransport::error("This always errors".to_owned()));
    Ok(())
  }
}
