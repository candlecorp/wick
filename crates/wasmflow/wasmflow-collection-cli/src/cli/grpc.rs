use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

use tokio::sync::mpsc::Sender;
use tonic::transport::{Certificate, Identity, Server};
use wasmflow_invocation_server::InvocationServer;
use wasmflow_rpc::rpc::invocation_service_server::InvocationServiceServer;

use super::{Result, ServerMessage};
use crate::options::ServerOptions;

pub(super) async fn start_rpc_server(
  options: &ServerOptions,
  svc: InvocationServiceServer<InvocationServer>,
) -> Result<(SocketAddr, Sender<ServerMessage>)> {
  debug!("Initializing RPC server");
  let port = options.port.unwrap_or(0);
  let address = options.address.unwrap_or(Ipv4Addr::from_str("127.0.0.1")?);

  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind(SocketAddr::new(IpAddr::V4(address), port))?;
  let addr = socket.local_addr()?;

  trace!(
    address = %addr,
    port = addr.port(),
    "RPC server address"
  );

  socket.set_reuseaddr(true).unwrap();
  #[cfg(not(target_os = "windows"))]
  socket.set_reuseport(true).unwrap();
  let listener = socket.listen(512).unwrap();

  let stream = tokio_stream::wrappers::TcpListenerStream::new(listener);

  #[cfg(feature = "reflection")]
  let reflection = tonic_reflection::server::Builder::configure()
    .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
    .build()
    .unwrap();

  let mut builder = Server::builder();

  if let (Some(pem), Some(key)) = (&options.pem, &options.key) {
    let server_pem = tokio::fs::read(pem).await?;
    let server_key = tokio::fs::read(key).await?;
    let identity = Identity::from_pem(server_pem, server_key);
    let mut tls = tonic::transport::ServerTlsConfig::new().identity(identity);

    if let Some(ca) = &options.ca {
      debug!(ca = %ca.to_string_lossy(),"RPC: Adding CA root");
      let ca_pem = tokio::fs::read(ca).await?;
      let ca = Certificate::from_pem(ca_pem);
      tls = tls.client_ca_root(ca);
    }

    builder = builder.tls_config(tls)?;
  } else if let Some(ca) = &options.ca {
    debug!(ca = %ca.to_string_lossy(),"RPC: Adding CA root");
    let ca_pem = tokio::fs::read(ca).await?;
    let ca = Certificate::from_pem(ca_pem);
    let tls = tonic::transport::ServerTlsConfig::new().client_ca_root(ca);
    builder = builder.tls_config(tls)?;
  }

  let inner = svc.clone();
  #[cfg(feature = "reflection")]
  let builder = builder.add_service(inner).add_service(reflection);
  #[cfg(not(feature = "reflection"))]
  let builder = builder.add_service(inner);

  let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerMessage>(1);
  let server = builder.serve_with_incoming_shutdown(stream, async move {
    rx.recv().await;
    info!("Received RPC shutdown message.");
  });

  tokio::spawn(async move {
    info!("Starting RPC server");
    if let Err(e) = server.await {
      error!("Error running RPC server: {}", e);
    };
    info!("RPC server shut down");
  });
  Ok((addr, tx))
}
