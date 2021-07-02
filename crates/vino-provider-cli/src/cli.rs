use std::net::{
  IpAddr,
  Ipv4Addr,
  SocketAddr,
};
use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::transport::Server;
use vino_rpc::rpc::invocation_service_server::InvocationServiceServer;
use vino_rpc::{
  InvocationServer,
  RpcHandler,
};

pub struct Options {
  pub port: Option<u16>,

  pub address: Ipv4Addr,
}

pub async fn init(
  provider: Arc<Mutex<dyn RpcHandler>>,
  opts: Option<Options>,
) -> crate::Result<SocketAddr> {
  debug!("Starting provider RPC server");

  let opts = match opts {
    Some(opts) => opts,
    None => Options {
      port: None,
      address: Ipv4Addr::from_str("127.0.0.1")?,
    },
  };

  let port = opts.port.unwrap_or(0);

  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind(SocketAddr::new(IpAddr::V4(opts.address), port))?;
  let addr = socket.local_addr()?;

  trace!("Binding to {:?}", addr.to_string());

  let component_service = InvocationServer { provider };

  let svc = InvocationServiceServer::new(component_service);

  let listener = tokio_stream::wrappers::TcpListenerStream::new(socket.listen(1).unwrap());
  tokio::spawn(
    Server::builder()
      .add_service(svc)
      .serve_with_incoming(listener),
  );

  trace!("Server started");

  Ok(addr)
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;
  use std::time::Duration;

  use test_vino_provider::Provider;
  use tonic::transport::Uri;
  use vino_rpc::make_rpc_client;
  use vino_rpc::rpc::ListRequest;

  use super::*;
  use crate::Result;

  #[test_env_log::test(tokio::test)]
  async fn test_starts() -> Result<()> {
    let ip = Ipv4Addr::from_str("127.0.0.1")?;
    let port = 12345;
    let addr = init(
      Arc::new(Mutex::new(Provider::default())),
      Some(Options {
        port: Some(port),
        address: Ipv4Addr::from_str("127.0.0.1")?,
      }),
    )
    .await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(addr.ip(), ip);
    assert_eq!(addr.port(), port);
    let uri = Uri::from_str(&format!("https://{}:{}", ip, port)).unwrap();
    let mut client = make_rpc_client(uri).await?;
    let response = client.list(ListRequest {}).await.unwrap();
    let list = response.into_inner();
    println!("list: {:?}", list);
    assert_eq!(list.components.len(), 1);
    Ok(())
  }
}
