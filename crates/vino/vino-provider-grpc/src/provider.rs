use std::time::Instant;

use async_trait::async_trait;
use http::Uri;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcClient, RpcHandler, RpcResult};

use crate::error::GrpcError;

#[derive(Debug)]
pub struct Provider {
  namespace: String,
  client: RpcClient,
  schemas: Vec<HostedType>,
}

impl Provider {
  pub async fn new<T: TryInto<Uri> + Send>(namespace: String, address: T) -> Result<Self, GrpcError> {
    let mut client = vino_rpc::make_rpc_client(address, None, None, None, None).await?;
    let schemas = client.list().await?;

    Ok(Self {
      namespace,
      client,
      schemas,
    })
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    let entity_url = entity.url();
    trace!("PROV:PAR:INVOKE:[{}]", entity_url);

    let start = Instant::now();

    let stream = self
      .client
      .clone()
      .invoke(self.namespace.clone(), entity.name(), payload)
      .await
      .map_err(|e| RpcError::ComponentError(e.to_string()))?;

    trace!(
      "PROV:PAR:INVOKE:[{}]:DURATION[{} ms]",
      entity_url,
      start.elapsed().as_millis()
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
    let service = Provider::new("test".to_owned(), addr).await?;

    let work = service.invoke(
      Entity::component_direct("test-component"),
      vec![("input", user_data)].into(),
    );

    tokio::select! {
        res = work => {
            debug!("Work complete");
            match res {
              Ok(mut response)=>{
                let next: TransportWrapper = response.next().await.unwrap();
                let payload: String = next.payload.try_into()?;
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
