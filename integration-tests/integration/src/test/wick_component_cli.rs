use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use flow_component::SharedComponent;
use tokio::time::sleep;
use tonic::transport::Uri;
use tracing::debug;
use wick_component_cli::options::{Options, ServerOptions};
use wick_component_cli::start_server;
use wick_invocation_server::connect_rpc_client;
use wick_rpc::rpc::ListRequest;

use super::NativeComponent;

fn get_component() -> SharedComponent {
  Arc::new(NativeComponent::default())
}

#[test_logger::test(tokio::test)]
async fn test_starts() -> Result<()> {
  let mut options = Options::default();
  let rpc_opts = ServerOptions {
    enabled: true,
    ..Default::default()
  };
  options.rpc = Some(rpc_opts);
  let config = start_server(get_component(), Some(options)).await?;
  let rpc = config.rpc.unwrap();
  debug!("Waiting for server to start");
  sleep(Duration::from_millis(100)).await;
  let uri = Uri::from_str(&format!("http://{}:{}", rpc.addr.ip(), rpc.addr.port())).unwrap();
  let mut client = connect_rpc_client(uri).await?;
  let response = client.list(ListRequest {}).await.unwrap();
  let list = response.into_inner();
  println!("list: {:?}", list);
  assert_eq!(list.components.len(), 1);
  Ok(())
}
