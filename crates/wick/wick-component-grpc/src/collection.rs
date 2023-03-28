use std::time::Instant;

use async_trait::async_trait;
use http::Uri;
use wasmflow_sdk::v1::transport::TransportStream;
use wasmflow_sdk::v1::Invocation;
use wick_interface_types::HostedType;
use wick_rpc::error::RpcError;
use wick_rpc::{RpcClient, RpcHandler, RpcResult};

use crate::error::GrpcError;

#[derive(Debug)]
pub struct Collection {
  client: RpcClient,
  schemas: Vec<HostedType>,
}

impl Collection {
  pub async fn new<T: TryInto<Uri> + Send>(address: T) -> Result<Self, GrpcError> {
    let mut client = wick_rpc::make_rpc_client(address, None, None, None, None).await?;
    let schemas = client.list().await?;

    Ok(Self { client, schemas })
  }
}

#[async_trait]
impl RpcHandler for Collection {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<TransportStream> {
    let target_url = invocation.target_url();
    trace!(target = %target_url, "grpc invoke");

    let start = Instant::now();

    let stream = self
      .client
      .clone()
      .invoke(invocation)
      .await
      .map_err(|e| RpcError::ComponentError(e.to_string()))?;

    trace!(
      target = %target_url,
      duration_ms = %start.elapsed().as_millis(),
      "grpc invoke complete",
    );
    Ok(stream)
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    Ok(self.schemas.clone())
  }
}

#[cfg(test)]
mod test {

  use std::sync::Arc;

  use anyhow::Result;
  use test_native_component::Collection as TestCollection;
  use wasmflow_sdk::v1::packet::PacketMap;
  use wick_invocation_server::{bind_new_socket, make_rpc_server};
  use wick_packet::Entity;
  use wick_rpc::SharedRpcHandler;

  use super::*;

  fn get_collection() -> SharedRpcHandler {
    Arc::new(TestCollection::default())
  }

  #[test_logger::test(tokio::test)]
  async fn test_initialize() -> Result<()> {
    let socket = bind_new_socket()?;
    let port = socket.local_addr()?.port();
    let init_handle = make_rpc_server(socket, get_collection());
    let user_data = "test string payload";
    let payload = PacketMap::from([("input", user_data)]);

    let addr = format!("http://127.0.0.1:{}", port);
    let service = Collection::new(addr).await?;
    let invocation = Invocation::new_test(file!(), Entity::local("test-component"), payload, None);

    let work = service.invoke(invocation);

    tokio::select! {
        res = work => {
            debug!("Work complete");
            match res {
              Ok(mut response)=>{
                let packets:Vec<_> = response.drain_port("output").await?;
                let next = packets[0].clone();
                let payload: String = next.payload.deserialize()?;
                assert_eq!(payload, format!("TEST: {}", user_data));
              },
              Err(e)=>{
                panic!("task failed: {}", e);
              }
            }
        }
        _ = init_handle => {
            panic!("server died");
        }
    };
    Ok(())
  }
}
