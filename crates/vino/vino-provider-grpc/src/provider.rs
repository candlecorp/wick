use std::time::Instant;

use async_trait::async_trait;
use http::Uri;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcClient, RpcHandler, RpcResult};
use vino_transport::Invocation;

use crate::error::GrpcError;

#[derive(Debug)]
pub struct Provider {
  client: RpcClient,
  schemas: Vec<HostedType>,
}

impl Provider {
  pub async fn new<T: TryInto<Uri> + Send>(address: T) -> Result<Self, GrpcError> {
    let mut client = vino_rpc::make_rpc_client(address, None, None, None, None).await?;
    let schemas = client.list().await?;

    Ok(Self { client, schemas })
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<BoxedTransportStream> {
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
    Ok(Box::pin(stream))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    Ok(self.schemas.clone())
  }
}

#[cfg(test)]
mod test {

  use std::sync::Arc;

  use anyhow::Result;
  use test_vino_provider::Provider as TestProvider;
  use tokio_stream::StreamExt;
  use vino_invocation_server::{bind_new_socket, make_rpc_server};
  use vino_rpc::SharedRpcHandler;

  use super::*;

  fn get_provider() -> SharedRpcHandler {
    Arc::new(TestProvider::default())
  }

  #[test_logger::test(tokio::test)]
  async fn test_initialize() -> Result<()> {
    let socket = bind_new_socket()?;
    let port = socket.local_addr()?.port();
    let init_handle = make_rpc_server(socket, get_provider());
    let user_data = "test string payload";

    let addr = format!("https://127.0.0.1:{}", port);
    let service = Provider::new(addr).await?;
    let invocation = Invocation::new_test(
      file!(),
      Entity::local_component("test-component"),
      vec![("input", user_data)].into(),
      None,
    );

    let work = service.invoke(invocation);

    tokio::select! {
        res = work => {
            debug!("Work complete");
            match res {
              Ok(mut response)=>{
                let next: TransportWrapper = response.next().await.unwrap();
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
