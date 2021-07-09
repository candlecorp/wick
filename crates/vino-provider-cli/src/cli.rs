use std::net::{
  IpAddr,
  Ipv4Addr,
  SocketAddr,
};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use tokio::signal;
use tokio::sync::Mutex;
use tonic::transport::{
  Certificate,
  Identity,
  Server,
};
use vino_rpc::rpc::invocation_service_server::InvocationServiceServer;
use vino_rpc::{
  InvocationServer,
  RpcHandler,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
  pub port: Option<u16>,

  pub address: Ipv4Addr,

  pub pem: Option<PathBuf>,

  pub key: Option<PathBuf>,

  pub ca: Option<PathBuf>,
}

pub async fn start_server(
  provider: Arc<Mutex<dyn RpcHandler>>,
  opts: Option<Options>,
) -> crate::Result<SocketAddr> {
  debug!("Starting provider RPC server");

  let opts = match opts {
    Some(opts) => opts,
    None => Options {
      port: None,
      address: Ipv4Addr::from_str("127.0.0.1")?,
      pem: None,
      key: None,
      ca: None,
    },
  };

  let port = opts.port.unwrap_or(0);

  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind(SocketAddr::new(IpAddr::V4(opts.address), port))?;
  let addr = socket.local_addr()?;

  trace!("Binding to {}", addr);

  let component_service = InvocationServer { provider };

  let svc = InvocationServiceServer::new(component_service);

  let listener = tokio_stream::wrappers::TcpListenerStream::new(socket.listen(1).unwrap());

  let reflection = tonic_reflection::server::Builder::configure()
    .build()
    .unwrap();

  // TODO: Need to decouple these options
  // they don't all need to be provided together
  let builder = if let (Some(pem), Some(key), Some(ca)) = (opts.pem, opts.key, opts.ca) {
    let server_pem = tokio::fs::read(pem).await?;
    let server_key = tokio::fs::read(key).await?;
    let ca_pem = tokio::fs::read(ca).await?;
    let ca = Certificate::from_pem(ca_pem);
    let identity = Identity::from_pem(server_pem, server_key);
    info!("Starting TLS server on {}", addr);
    let tls = tonic::transport::ServerTlsConfig::new()
      .identity(identity)
      .client_ca_root(ca);

    Server::builder()
      .tls_config(tls)?
      .add_service(reflection)
      .add_service(svc)
      .serve_with_incoming(listener)
  } else {
    info!("Starting insecure server on {}", addr);
    Server::builder()
      .add_service(reflection)
      .add_service(svc)
      .serve_with_incoming(listener)
  };
  tokio::spawn(builder);

  Ok(addr)
}

pub async fn init_cli(
  provider: Arc<Mutex<dyn RpcHandler>>,
  opts: Option<Options>,
) -> crate::Result<()> {
  let addr = start_server(provider, opts).await?;
  info!("Server bound to {}", addr);
  info!("Waiting for ctrl-C");
  signal::ctrl_c().await?;

  Ok(())
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
    let addr = start_server(
      Arc::new(Mutex::new(Provider::default())),
      Some(Options {
        port: Some(port),
        address: Ipv4Addr::from_str("127.0.0.1")?,
        pem: None,
        ca: None,
        key: None,
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
