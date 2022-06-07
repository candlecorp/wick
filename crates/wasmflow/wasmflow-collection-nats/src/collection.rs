use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use futures::executor::block_on;
use wasmflow_invocation::Invocation;
use wasmflow_mesh::Mesh;
use wasmflow_rpc::error::RpcError;
use wasmflow_rpc::{RpcHandler, RpcResult};
use wasmflow_transport::TransportStream;

use crate::Error;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Collection {
  mesh_id: String,
  mesh: Arc<Mesh>,
}

impl Collection {
  pub async fn new(mesh_id: String, mesh: Arc<Mesh>) -> Result<Self, Error> {
    Ok(Self { mesh_id, mesh })
  }
}

#[async_trait]
impl RpcHandler for Collection {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<TransportStream> {
    let target_url = invocation.target_url();
    trace!(target = %target_url, "mesh invoke");

    let start = Instant::now();
    let stream = self
      .mesh
      .invoke(&self.mesh_id, invocation)
      .await
      .map_err(|e| RpcError::CollectionError(e.to_string()))?;

    trace!(
      target = %target_url,
      duration_ms = %start.elapsed().as_millis(),
      "response stream received",
    );

    Ok(stream)
  }

  fn get_list(&self) -> RpcResult<Vec<wasmflow_interface::HostedType>> {
    let components = block_on(self.mesh.list_components(self.mesh_id.clone()))
      .map_err(|e| RpcError::CollectionError(e.to_string()))?;

    Ok(components)
  }
}

#[cfg(test)]
mod tests {

  use anyhow::Result as TestResult;
  use wasmflow_entity::Entity;
  use wasmflow_mesh::MeshBuilder;
  use wasmflow_packet::PacketMap;
  use wasmflow_rpc::SharedRpcHandler;
  use wasmflow_transport::MessageTransport;

  use super::*;

  fn get_collection() -> SharedRpcHandler {
    Arc::new(test_native_collection::Collection::default())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_component() -> TestResult<()> {
    let mesh_builder = MeshBuilder::new_from_env("test")?;
    let mesh = mesh_builder.build().await?;
    let ns = "some_namespace";

    mesh.handle_namespace(ns.to_owned(), get_collection()).await?;

    let collection = Collection::new(ns.to_owned(), Arc::new(mesh)).await?;
    let user_data = "Hello world";

    let job_payload = PacketMap::from([("input", user_data)]);
    let invocation = Invocation::new_test(file!(), Entity::component(ns, "test-component"), job_payload, None);

    let mut stream = collection.invoke(invocation).await?;
    let output = stream.drain_port("output").await?[0].clone();

    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output: String = output.payload.deserialize()?;

    println!("output: {:?}", output);
    assert_eq!(output, format!("TEST: {}", user_data));
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_error() -> TestResult<()> {
    let mesh_builder = MeshBuilder::new_from_env("test")?;
    let mesh = mesh_builder.build().await?;
    let ns = "some_namespace";

    mesh.handle_namespace(ns.to_owned(), get_collection()).await?;

    let collection = Collection::new(ns.to_owned(), Arc::new(mesh)).await?;
    let user_data = "Hello world";

    let job_payload = PacketMap::from([("input", user_data)]);

    let invocation = Invocation::new_test(file!(), Entity::component(ns, "error"), job_payload, None);

    let mut stream = collection.invoke(invocation).await?;
    let outputs = stream.drain().await;
    let output = outputs[0].clone();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    assert_eq!(output.payload, MessageTransport::error("This always errors"));
    Ok(())
  }
}
