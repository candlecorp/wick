use std::net::{
  IpAddr,
  Ipv4Addr,
  SocketAddr,
};
use std::path::PathBuf;
use std::str::FromStr;

use logger::LoggingOptions;
use structopt::StructOpt;
use tokio::signal;
use tonic::transport::{
  Certificate,
  Identity,
  Server,
};
use vino_invocation_server::InvocationServer;
use vino_rpc::rpc::invocation_service_server::InvocationServiceServer;
use vino_rpc::BoxedRpcHandler;

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
  include_bytes!("../../vino-rpc/src/generated/descriptors.bin");

#[derive(Debug, Default, Clone)]
/// Server configuration options.
pub struct Options {
  /// RPC server options.
  pub rpc: Option<ServerOptions>,
  /// HTTP server options.
  pub http: Option<ServerOptions>,
}

#[derive(Debug, Clone, Default)]
/// Options to use when starting an RPC or HTTP server.
pub struct ServerOptions {
  /// The port to bind to.
  pub port: Option<u16>,

  /// The address to bind to.
  pub address: Option<Ipv4Addr>,

  /// Path to pem file for TLS.
  pub pem: Option<PathBuf>,

  /// Path to key file for TLS.
  pub key: Option<PathBuf>,

  /// Path to CA file.
  pub ca: Option<PathBuf>,
}

impl From<DefaultCliOptions> for Options {
  fn from(opts: DefaultCliOptions) -> Self {
    Options {
      rpc: Some(ServerOptions {
        port: opts.port,
        address: opts.address,
        pem: opts.pem,
        key: opts.key,
        ca: opts.ca,
      }),
      http: Some(ServerOptions {
        port: opts.http_port,
        address: opts.http_address,
        pem: None,
        key: None,
        ca: None,
      }),
    }
  }
}

impl From<DefaultCliOptions> for LoggingOptions {
  fn from(opts: DefaultCliOptions) -> Self {
    opts.logging
  }
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
/// Default command line options that downstream providers can use. This isn't required.
pub struct DefaultCliOptions {
  /// Port to listen on.
  #[structopt(short, long)]
  pub port: Option<u16>,

  /// IP address to bind to.
  #[structopt(short, long)]
  pub address: Option<Ipv4Addr>,

  /// Path to pem file for TLS.
  #[structopt(long)]
  pub pem: Option<PathBuf>,

  /// Path to key file for TLS.
  #[structopt(long)]
  pub key: Option<PathBuf>,

  /// Path to certificate authority.
  #[structopt(long)]
  pub ca: Option<PathBuf>,

  /// Address for the optional HTTP server.
  #[structopt(long)]
  pub http_address: Option<Ipv4Addr>,

  /// Port to use for HTTP.
  #[structopt(long)]
  pub http_port: Option<u16>,

  /// Logging options.
  #[structopt(flatten)]
  pub logging: LoggingOptions,

  /// Outputs the version.
  #[structopt(long = "version", short = "v")]
  pub version: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
/// Metadata for the running server.
pub struct ServerMetadata {
  /// The address of the RPC server if it is running.
  pub rpc_addr: Option<SocketAddr>,
  /// The address of the HTTP server if it is running.
  pub http_addr: Option<SocketAddr>,
}

/// Initializes logging with Vino's logger.
pub fn init_logging(options: &LoggingOptions) -> crate::Result<()> {
  logger::init(options);

  Ok(())
}

/// Starts an RPC and/or an HTTP server for the passed [vino_rpc::RpcHandler].
pub async fn start_server(
  provider: BoxedRpcHandler,
  opts: Option<Options>,
) -> crate::Result<ServerMetadata> {
  debug!("Starting RPC server");

  let opts = opts.unwrap_or_default();

  let rpc_options = opts.rpc.unwrap_or_default();

  let port = rpc_options.port.unwrap_or(0);
  let address = rpc_options
    .address
    .unwrap_or(Ipv4Addr::from_str("127.0.0.1")?);

  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind(SocketAddr::new(IpAddr::V4(address), port))?;
  let addr = socket.local_addr()?;

  trace!("Binding RPC server to {} (Port: {})", addr, addr.port());

  let component_service = InvocationServer::new(provider);

  let svc = InvocationServiceServer::new(component_service);

  let listener = tokio_stream::wrappers::TcpListenerStream::new(socket.listen(512).unwrap());

  let reflection = tonic_reflection::server::Builder::configure()
    .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
    .build()
    .unwrap();

  let mut builder = Server::builder();

  if let (Some(pem), Some(key)) = (rpc_options.pem, rpc_options.key) {
    let server_pem = tokio::fs::read(pem).await?;
    let server_key = tokio::fs::read(key).await?;
    let identity = Identity::from_pem(server_pem, server_key);
    info!("Starting secure server on {}", addr);
    let mut tls = tonic::transport::ServerTlsConfig::new().identity(identity);

    if let Some(ca) = rpc_options.ca {
      debug!("Adding CA root from {}", ca.to_string_lossy());
      let ca_pem = tokio::fs::read(ca).await?;
      let ca = Certificate::from_pem(ca_pem);
      tls = tls.client_ca_root(ca);
    }

    builder = builder.tls_config(tls)?;
  } else {
    if let Some(ca) = rpc_options.ca {
      debug!("Adding CA root from {}", ca.to_string_lossy());
      let ca_pem = tokio::fs::read(ca).await?;
      let ca = Certificate::from_pem(ca_pem);
      let tls = tonic::transport::ServerTlsConfig::new().client_ca_root(ca);
      builder = builder.tls_config(tls)?;
    }

    info!("Starting insecure server on {}", addr);
  }

  let http_addr = if let Some(http_opts) = opts.http {
    let port = http_opts.port.unwrap_or(0);
    let address = http_opts
      .address
      .unwrap_or(Ipv4Addr::from_str("127.0.0.1")?);

    let socket = tokio::net::TcpSocket::new_v4()?;
    socket.bind(SocketAddr::new(IpAddr::V4(address), port))?;
    let addr = socket.local_addr()?;

    let listener = tokio_stream::wrappers::TcpListenerStream::new(socket.listen(512).unwrap());

    let web_service = tonic_web::config().allow_all_origins().enable(svc.clone());

    let server = Server::builder()
      .accept_http1(true)
      .add_service(web_service)
      .serve_with_incoming(listener);

    tokio::spawn(server);

    Some(addr)
  } else {
    None
  };

  let builder = builder.add_service(reflection).add_service(svc);

  let server = builder.serve_with_incoming(listener);

  tokio::spawn(server);

  Ok(ServerMetadata {
    rpc_addr: Some(addr),
    http_addr,
  })
}

/// Start a server with the passed [vino_rpc::RpcHandler] and keep it.
/// running until the process receives a SIGINT (^C).
pub async fn init_cli(provider: BoxedRpcHandler, opts: Option<Options>) -> crate::Result<()> {
  let _ = start_server(provider, opts).await?;
  info!("Waiting for ctrl-C");
  signal::ctrl_c().await?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;
  use std::time::Duration;

  use anyhow::Result;
  use test_vino_provider::Provider;
  use tokio::time::sleep;
  use tonic::transport::Uri;
  use vino_invocation_server::make_rpc_client;
  use vino_rpc::rpc::ListRequest;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_starts() -> Result<()> {
    let config = start_server(Box::new(Provider::default()), None).await?;
    let addr = config.rpc_addr.unwrap();
    sleep(Duration::from_millis(100)).await;
    let uri = Uri::from_str(&format!("https://{}:{}", addr.ip(), addr.port())).unwrap();
    let mut client = make_rpc_client(uri).await?;
    let response = client.list(ListRequest {}).await.unwrap();
    let list = response.into_inner();
    println!("list: {:?}", list);
    assert_eq!(list.components.len(), 1);
    Ok(())
  }

  // #[test_logger::test(tokio::test)]
  async fn _test_http() -> Result<()> {
    let config = start_server(
      Box::new(Provider::default()),
      Some(Options {
        rpc: Some(ServerOptions {
          address: Some(Ipv4Addr::from_str("127.0.0.1")?),
          port: Some(8112),
          ..Default::default()
        }),
        http: Some(ServerOptions {
          address: Some(Ipv4Addr::from_str("127.0.0.1")?),
          port: Some(8111),
          ..Default::default()
        }),
      }),
    )
    .await?;
    sleep(Duration::from_millis(100)).await;
    let addr = config.http_addr.unwrap();
    let url = &format!("http://{}:{}", addr.ip(), addr.port());
    println!("URL: {}", url);
    // sleep(Duration::from_millis(1000000)).await;
    let endpoint = format!("{}/vino.InvocationService.List", url);
    let client = reqwest::Client::new();
    let resp = client
      .post(endpoint)
      .header("accept", "application/json")
      .header("Content-Type", "application/json")
      .body("{}")
      .send()
      .await?;

    println!("http response: {:?}", resp);
    // todo!();
    Ok(())
  }
}
