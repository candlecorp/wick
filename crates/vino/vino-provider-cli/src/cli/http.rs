use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  str::FromStr,
};

use tokio::sync::mpsc::Sender;
use tonic::transport::Server;
use vino_rpc::SharedRpcHandler;

use crate::options::ServerOptions;

use super::{Result, ServerMessage};

pub(super) async fn start_http_server(
  options: &ServerOptions,
  provider: SharedRpcHandler,
) -> Result<(SocketAddr, Sender<ServerMessage>)> {
  let port = options.port.unwrap_or(0);
  let address = options.address.unwrap_or(Ipv4Addr::from_str("127.0.0.1")?);

  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind(SocketAddr::new(IpAddr::V4(address), port))?;
  let addr = socket.local_addr()?;

  trace!("HTTP: Starting server on {}", addr);

  socket.set_reuseaddr(true).unwrap();
  #[cfg(not(target_os = "windows"))]
  socket.set_reuseport(true).unwrap();
  let listener = socket.listen(512).unwrap();

  let stream = tokio_stream::wrappers::TcpListenerStream::new(listener);

  let web_service = vino_http::config().allow_all_origins().enable(provider);

  if options.ca.is_some() || options.pem.is_some() || options.key.is_some() {
    info!(
      "HTTPS server is temporarily disabled and serving requests over HTTP1 is for testing only."
    );
  }

  let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerMessage>(1);
  let server = Server::builder()
    .accept_http1(true)
    .add_service(web_service)
    .serve_with_incoming_shutdown(stream, async move {
      rx.recv().await;
      info!("Shut down HTTP server.");
    });

  tokio::spawn(server);

  Ok((addr, tx))
}
