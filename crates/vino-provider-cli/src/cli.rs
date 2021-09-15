use std::net::{
  IpAddr,
  Ipv4Addr,
  SocketAddr,
};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use logger::LoggingOptions;
use nkeys::KeyPair;
use structopt::StructOpt;
use tokio::signal;
use tonic::transport::{
  Certificate,
  Identity,
  Server,
};
use vino_invocation_server::InvocationServer;
use vino_lattice::lattice::Lattice;
use vino_lattice::nats::NatsOptions;
use vino_rpc::rpc::invocation_service_server::InvocationServiceServer;
use vino_rpc::RpcFactory;

use crate::Result;

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
  include_bytes!("../../vino-rpc/src/generated/descriptors.bin");

#[derive(Debug, Clone)]
/// Server configuration options.
pub struct Options {
  /// RPC server options.
  pub rpc: Option<ServerOptions>,
  /// HTTP server options.
  pub http: Option<ServerOptions>,
  /// Lattice options.
  pub lattice: Option<LatticeOptions>,
  /// The ID of the server.
  pub id: String,
  /// The timeout for network requests.
  pub timeout: Duration,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      id: KeyPair::new_server().public_key(),
      rpc: Default::default(),
      http: Default::default(),
      lattice: Default::default(),
      timeout: Default::default(),
    }
  }
}

#[derive(Debug, Default, Clone)]
/// Configuration used to connect to the lattice
pub struct LatticeOptions {
  /// Enable/disable the lattice connection.
  pub enabled: bool,

  /// The address of the NATS server.
  pub address: String,

  /// The path to the NATS credsfile.
  pub creds_path: Option<PathBuf>,

  /// The NATS token.
  pub token: Option<String>,
}

#[derive(Debug, Clone, Default)]
/// Options to use when starting an RPC or HTTP server.
pub struct ServerOptions {
  /// Enable/disable the server.
  pub enabled: bool,

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
    let rpc = Some(ServerOptions {
      enabled: opts.rpc_enabled,
      port: opts.rpc_port,
      address: opts.rpc_address,
      pem: opts.rpc_pem,
      key: opts.rpc_key,
      ca: opts.rpc_ca,
    });

    let http = Some(ServerOptions {
      enabled: opts.http_enabled,
      port: opts.http_port,
      address: opts.http_address,
      pem: opts.http_pem,
      key: opts.http_key,
      ca: opts.http_ca,
    });

    #[allow(clippy::option_if_let_else)]
    let lattice = if let Some(url) = opts.nats_url {
      Some(LatticeOptions {
        enabled: opts.lattice_enabled,
        address: url,
        creds_path: opts.nats_credsfile,
        token: opts.nats_token,
      })
    } else {
      None
    };

    Options {
      rpc,
      http,
      timeout: Duration::from_millis(opts.timeout.unwrap_or(5000)),
      id: opts
        .id
        .unwrap_or_else(|| KeyPair::new_server().public_key()),
      lattice,
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
/// Command line options for providers.
pub struct DefaultCliOptions {
  /// The unique ID of this client.
  #[structopt(long = "id", env = "PROVIDER_ID")]
  pub id: Option<String>,

  /// The timeout for outbound requests in ms.
  #[structopt(long = "timeout", env = "VINO_TIMEOUT")]
  pub timeout: Option<u64>,

  /// Logging options.
  #[structopt(flatten)]
  pub logging: LoggingOptions,

  /// Outputs the version.
  #[structopt(long = "version", short = "v")]
  pub version: bool,

  /// Enable the lattice connection.
  #[structopt(long = "lattice")]
  pub lattice_enabled: bool,

  /// The url of the NATS server (in IP:PORT format).
  #[structopt(long = "nats", env = "NATS_URL")]
  pub nats_url: Option<String>,

  /// The path to the NATS credsfile.
  #[structopt(long = "nats-credsfile", env = "NATS_CREDSFILE")]
  pub nats_credsfile: Option<PathBuf>,

  /// The NATS token.
  #[structopt(long = "nats-token", env = "NATS_TOKEN", hide_env_values = true)]
  pub nats_token: Option<String>,

  /// Enable the rpc server.
  #[structopt(long = "rpc")]
  pub rpc_enabled: bool,

  /// Port to listen on for GRPC server.
  #[structopt(long = "rpc-port", env = "VINO_RPC_PORT")]
  pub rpc_port: Option<u16>,

  /// IP address to bind to for GRPC server.
  #[structopt(long = "rpc-address", env = "VINO_RPC_ADDRESS")]
  pub rpc_address: Option<Ipv4Addr>,

  /// Path to pem file for TLS for GRPC server.
  #[structopt(long = "rpc-pem", env = "VINO_RPC_PEM")]
  pub rpc_pem: Option<PathBuf>,

  /// Path to key file for TLS for GRPC server.
  #[structopt(long = "rpc-key", env = "VINO_RPC_KEY")]
  pub rpc_key: Option<PathBuf>,

  /// Path to certificate authority for GRPC server.
  #[structopt(long = "rpc-ca", env = "VINO_RPC_CA")]
  pub rpc_ca: Option<PathBuf>,

  /// Enable the http server.
  #[structopt(long = "http")]
  pub http_enabled: bool,

  /// Address for the optional HTTP server.
  #[structopt(long = "http-address", env = "VINO_HTTP_ADDRESS")]
  pub http_address: Option<Ipv4Addr>,

  /// Port to use for HTTP.
  #[structopt(long = "http-port", env = "VINO_HTTP_PORT")]
  pub http_port: Option<u16>,

  /// Path to pem file for TLS for HTTPS server.
  #[structopt(long = "http-pem", env = "VINO_HTTP_PEM")]
  pub http_pem: Option<PathBuf>,

  /// Path to key file for TLS for HTTPS server.
  #[structopt(long = "http-key", env = "VINO_HTTP_KEY")]
  pub http_key: Option<PathBuf>,

  /// Path to certificate authority for HTTPS server.
  #[structopt(long = "http-ca", env = "VINO_HTTP_CA")]
  pub http_ca: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
#[must_use]
/// Metadata for the running server.
pub struct ServerMetadata {
  /// The address of the RPC server if it is running.
  pub rpc_addr: Option<SocketAddr>,
  /// The address of the HTTP server if it is running.
  pub http_addr: Option<SocketAddr>,
  /// True if we're connected to the lattice, false otherwise.
  pub lattice_connected: bool,
  /// The ID of the server.
  pub id: String,
}

#[doc(hidden)]
pub fn print_info(info: &ServerMetadata) {
  let mut something_started = false;
  if let Some(addr) = info.rpc_addr {
    something_started = true;
    info!("GRPC server bound to {} on port {}", addr.ip(), addr.port());
  }
  if let Some(addr) = info.http_addr {
    something_started = true;
    info!("HTTP server bound to {} on port {}", addr.ip(), addr.port());
  }
  if info.lattice_connected {
    something_started = true;
    info!("Host connected to lattice with id {}", info.id);
  }
  if !something_started {
    warn!("No server information available, did you intend to start a host without GRPC or a lattice connection?");
    warn!("If not, try passing the flag --rpc or --lattice to explicitly enable those features.");
  }
}

/// Initializes logging with Vino's logger.
pub fn init_logging(options: &LoggingOptions) -> Result<()> {
  logger::init(options);

  Ok(())
}

/// Starts an RPC and/or an HTTP server for the passed [vino_rpc::RpcHandler].
pub async fn start_server(provider: RpcFactory, opts: Option<Options>) -> Result<ServerMetadata> {
  debug!("Starting server with options: {:?}", opts);

  let opts = opts.unwrap_or_default();

  let component_service = InvocationServer::new(provider());

  let svc = InvocationServiceServer::new(component_service);

  let rpc_addr = if let Some(rpc_options) = opts.rpc {
    if !rpc_options.enabled {
      None
    } else {
      let addr = start_rpc_server(&rpc_options, svc.clone()).await?;
      Some(addr)
    }
  } else {
    None
  };

  let http_addr = if let Some(http_opts) = opts.http {
    if !http_opts.enabled {
      None
    } else {
      let addr = start_http_server(&http_opts, svc.clone()).await?;
      Some(addr)
    }
  } else {
    None
  };

  let lattice_connected = match &opts.lattice {
    Some(lattice) => {
      if lattice.enabled {
        connect_to_lattice(lattice, opts.id.clone(), provider, opts.timeout).await?;
        true
      } else {
        false
      }
    }
    None => false,
  };

  Ok(ServerMetadata {
    id: opts.id,
    rpc_addr,
    http_addr,
    lattice_connected,
  })
}

async fn connect_to_lattice(
  opts: &LatticeOptions,
  id: String,
  factory: RpcFactory,
  timeout: Duration,
) -> Result<()> {
  info!("Connecting to lattice...");
  let lattice = Lattice::connect(NatsOptions {
    address: opts.address.clone(),
    client_id: id.clone(),
    creds_path: opts.creds_path.clone(),
    token: opts.token.clone(),
    timeout,
  })
  .await?;
  info!("Registering '{}' on lattice...", id);
  lattice.handle_namespace(id, factory).await?;
  Ok(())
}

async fn start_http_server(
  options: &ServerOptions,
  svc: InvocationServiceServer<InvocationServer>,
) -> Result<SocketAddr> {
  info!("Starting HTTP server");
  let port = options.port.unwrap_or(0);
  let address = options.address.unwrap_or(Ipv4Addr::from_str("127.0.0.1")?);

  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind(SocketAddr::new(IpAddr::V4(address), port))?;
  let addr = socket.local_addr()?;

  let listener = tokio_stream::wrappers::TcpListenerStream::new(socket.listen(512).unwrap());

  let web_service = tonic_web::config().allow_all_origins().enable(svc);

  if options.ca.is_some() || options.pem.is_some() || options.key.is_some() {
    info!(
      "HTTPS server is temporarily disabled and serving requests over HTTP1 is for testing only."
    );
  }

  info!("HTTP: Starting insecure server on {}", addr);
  let server = Server::builder()
    .accept_http1(true)
    .add_service(web_service)
    .serve_with_incoming(listener);

  tokio::spawn(server);

  Ok(addr)
}

async fn start_rpc_server(
  options: &ServerOptions,
  svc: InvocationServiceServer<InvocationServer>,
) -> Result<SocketAddr> {
  info!("Starting RPC server");
  let port = options.port.unwrap_or(0);
  let address = options.address.unwrap_or(Ipv4Addr::from_str("127.0.0.1")?);

  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind(SocketAddr::new(IpAddr::V4(address), port))?;
  let addr = socket.local_addr()?;

  trace!("Binding RPC server to {} (Port: {})", addr, addr.port());

  let listener = tokio_stream::wrappers::TcpListenerStream::new(socket.listen(512).unwrap());

  let reflection = tonic_reflection::server::Builder::configure()
    .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
    .build()
    .unwrap();

  let mut builder = Server::builder();

  if let (Some(pem), Some(key)) = (&options.pem, &options.key) {
    let server_pem = tokio::fs::read(pem).await?;
    let server_key = tokio::fs::read(key).await?;
    let identity = Identity::from_pem(server_pem, server_key);
    info!("RPC: Starting secure server on {}", addr);
    let mut tls = tonic::transport::ServerTlsConfig::new().identity(identity);

    if let Some(ca) = &options.ca {
      debug!("RPC: Adding CA root from {}", ca.to_string_lossy());
      let ca_pem = tokio::fs::read(ca).await?;
      let ca = Certificate::from_pem(ca_pem);
      tls = tls.client_ca_root(ca);
    }

    builder = builder.tls_config(tls)?;
  } else {
    if let Some(ca) = &options.ca {
      debug!("RPC: Adding CA root from {}", ca.to_string_lossy());
      let ca_pem = tokio::fs::read(ca).await?;
      let ca = Certificate::from_pem(ca_pem);
      let tls = tonic::transport::ServerTlsConfig::new().client_ca_root(ca);
      builder = builder.tls_config(tls)?;
    }

    info!("RPC: Starting insecure server on {}", addr);
  }

  let inner = svc.clone();
  let builder = builder.add_service(reflection).add_service(inner);

  let server = builder.serve_with_incoming(listener);

  tokio::spawn(server);
  Ok(addr)
}

/// Start a server with the passed [vino_rpc::RpcHandler] and keep it.
/// running until the process receives a SIGINT (^C).
pub async fn init_cli(provider: RpcFactory, opts: Option<Options>) -> Result<()> {
  let info = start_server(provider, opts).await?;
  print_info(&info);

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
    let mut options = Options::default();
    let rpc_opts = ServerOptions {
      enabled: true,
      ..Default::default()
    };
    options.rpc = Some(rpc_opts);
    let config = start_server(Box::new(|| Box::new(Provider::default())), Some(options)).await?;
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
      Box::new(|| Box::new(Provider::default())),
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
        ..Default::default()
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
