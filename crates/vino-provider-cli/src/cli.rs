use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use logger::LoggingOptions;
use nkeys::KeyPair;
use structopt::StructOpt;
use tokio::signal;
use tokio::sync::mpsc::Sender;
use tonic::transport::{Certificate, Identity, Server};
use vino_invocation_server::InvocationServer;
use vino_lattice::lattice::Lattice;
use vino_lattice::nats::NatsOptions;
use vino_rpc::rpc::invocation_service_server::InvocationServiceServer;
use vino_rpc::SharedRpcHandler;

use crate::Result;

#[cfg(feature = "reflection")]
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
    let lattice = if let Some(url) = opts.lattice.nats_url {
      Some(LatticeOptions {
        enabled: opts.lattice.lattice_enabled,
        address: url,
        creds_path: opts.lattice.nats_credsfile,
        token: opts.lattice.nats_token,
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

#[derive(Debug, Clone, Default, StructOpt)]
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

  #[structopt(flatten)]
  /// Options for connecting to a lattice.
  pub lattice: LatticeCliOptions,

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

#[derive(Debug, Clone, Default, StructOpt)]
/// Command line options for providers.
pub struct LatticeCliOptions {
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
}

#[derive(Debug)]
#[must_use]
/// Metadata for the running server.
pub struct ServerState {
  /// The address of the RPC server if it is running.
  pub rpc: Option<ServerControl>,
  /// The address of the HTTP server if it is running.
  pub http: Option<ServerControl>,

  /// True if we're connected to the lattice, false otherwise.
  pub lattice: Option<Arc<Lattice>>,
  /// The ID of the server.
  pub id: String,
}

/// Struct that holds control methods and metadata for a running service.
pub struct ServerControl {
  /// The address of the RPC server.
  pub addr: SocketAddr,
  tx: Sender<ServerMessage>,
}

impl std::fmt::Debug for ServerControl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ServerControl")
      .field("addr", &self.addr)
      .finish()
  }
}

impl ServerControl {
  fn maybe_new(opt: Option<(SocketAddr, Sender<ServerMessage>)>) -> Option<Self> {
    if let Some((addr, tx)) = opt {
      Some(Self { addr, tx })
    } else {
      None
    }
  }
}

impl ServerState {
  /// Stop the RPC server if it's running.
  pub fn stop_rpc_server(&self) {
    if let Some(ctl) = self.rpc.as_ref() {
      let _ = ctl.tx.send(ServerMessage::Close);
    }
  }
  /// Stop the HTTP server if it's running.
  pub fn stop_http_server(&self) {
    if let Some(ctl) = &self.http {
      let _ = ctl.tx.send(ServerMessage::Close);
    }
  }
}

#[doc(hidden)]
pub fn print_info(info: &ServerState) {
  let mut something_started = false;
  if let Some(addr) = &info.rpc {
    let addr = addr.addr;
    something_started = true;
    info!("GRPC server bound to {} on port {}", addr.ip(), addr.port());
  }
  if let Some(addr) = &info.http {
    let addr = addr.addr;
    something_started = true;
    info!("HTTP server bound to {} on port {}", addr.ip(), addr.port());
  }

  if info.lattice.is_some() {
    something_started = true;
    info!("Host connected to lattice with id '{}'", info.id);
  }
  if !something_started {
    warn!("No server information available, did you intend to start a host without GRPC or a lattice connection?");
    warn!("If not, try passing the flag --rpc or --lattice to explicitly enable those features.");
  }
}

/// Starts an RPC and/or an HTTP server for the passed [vino_rpc::RpcHandler].
pub async fn start_server(
  provider: SharedRpcHandler,
  opts: Option<Options>,
) -> Result<ServerState> {
  debug!("Starting server with options: {:?}", opts);

  let opts = opts.unwrap_or_default();

  let component_service = InvocationServer::new(provider.clone());

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
      let addr = start_http_server(&http_opts, provider.clone()).await?;
      Some(addr)
    }
  } else {
    None
  };

  let lattice = match &opts.lattice {
    Some(lattice) => {
      if lattice.enabled {
        let lattice = connect_to_lattice(lattice, opts.id.clone(), provider, opts.timeout).await?;
        Some(lattice)
      } else {
        None
      }
    }
    None => None,
  };

  Ok(ServerState {
    id: opts.id,
    rpc: ServerControl::maybe_new(rpc_addr),
    http: ServerControl::maybe_new(http_addr),

    lattice,
  })
}

async fn connect_to_lattice(
  opts: &LatticeOptions,
  id: String,
  factory: SharedRpcHandler,
  timeout: Duration,
) -> Result<Arc<Lattice>> {
  info!(
    "Connecting to lattice at {} with timeout of {} ms...",
    opts.address,
    timeout.as_millis()
  );
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
  Ok(Arc::new(lattice))
}

async fn start_http_server(
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

enum ServerMessage {
  Close,
}

async fn start_rpc_server(
  options: &ServerOptions,
  svc: InvocationServiceServer<InvocationServer>,
) -> Result<(SocketAddr, Sender<ServerMessage>)> {
  info!("Starting RPC server");
  let port = options.port.unwrap_or(0);
  let address = options.address.unwrap_or(Ipv4Addr::from_str("127.0.0.1")?);

  let socket = tokio::net::TcpSocket::new_v4()?;
  socket.bind(SocketAddr::new(IpAddr::V4(address), port))?;
  let addr = socket.local_addr()?;

  trace!("Binding RPC server to {} (Port: {})", addr, addr.port());

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

  trace!("RPC: Starting server on {}", addr);
  if let (Some(pem), Some(key)) = (&options.pem, &options.key) {
    let server_pem = tokio::fs::read(pem).await?;
    let server_key = tokio::fs::read(key).await?;
    let identity = Identity::from_pem(server_pem, server_key);
    let mut tls = tonic::transport::ServerTlsConfig::new().identity(identity);

    if let Some(ca) = &options.ca {
      debug!("RPC: Adding CA root from {}", ca.to_string_lossy());
      let ca_pem = tokio::fs::read(ca).await?;
      let ca = Certificate::from_pem(ca_pem);
      tls = tls.client_ca_root(ca);
    }

    builder = builder.tls_config(tls)?;
  } else if let Some(ca) = &options.ca {
    debug!("RPC: Adding CA root from {}", ca.to_string_lossy());
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
    info!("Shut down RPC server.");
  });

  tokio::spawn(server);
  Ok((addr, tx))
}

/// Start a server with the passed [vino_rpc::RpcHandler] and keep it.
/// running until the process receives a SIGINT (^C).
pub async fn init_cli(provider: SharedRpcHandler, opts: Option<Options>) -> Result<()> {
  let state = start_server(provider, opts).await?;
  print_info(&state);

  info!("Waiting for ctrl-C");
  signal::ctrl_c().await?;
  println!(); // start on a new line.
  state.stop_http_server();
  state.stop_rpc_server();

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;
  use std::sync::Arc;
  use std::time::Duration;

  use anyhow::Result;
  use test_vino_provider::Provider;
  use tokio::time::sleep;
  use tonic::transport::Uri;
  use vino_invocation_server::connect_rpc_client;
  use vino_rpc::rpc::ListRequest;

  use super::*;

  fn get_provider() -> SharedRpcHandler {
    Arc::new(Provider::default())
  }

  #[test_logger::test(tokio::test)]
  async fn test_starts() -> Result<()> {
    let mut options = Options::default();
    let rpc_opts = ServerOptions {
      enabled: true,
      ..Default::default()
    };
    options.rpc = Some(rpc_opts);
    let config = start_server(get_provider(), Some(options)).await?;
    let rpc = config.rpc.unwrap();
    debug!("Waiting for server to start");
    sleep(Duration::from_millis(100)).await;
    let uri = Uri::from_str(&format!("https://{}:{}", rpc.addr.ip(), rpc.addr.port())).unwrap();
    let mut client = connect_rpc_client(uri).await?;
    let response = client.list(ListRequest {}).await.unwrap();
    let list = response.into_inner();
    println!("list: {:?}", list);
    assert_eq!(list.schemas.len(), 1);
    Ok(())
  }

  // #[test_logger::test(tokio::test)]
  async fn _test_http() -> Result<()> {
    let config = start_server(
      get_provider(),
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
    let http = config.http.unwrap();
    let url = &format!("http://{}:{}", http.addr.ip(), http.addr.port());
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
