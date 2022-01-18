use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::time::Duration;

use logger::LoggingOptions;
use nkeys::KeyPair;
use structopt::StructOpt;

#[derive(Debug)]
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

  /// Enable RPC over STDIO
  #[structopt(long, env = "VINO_STDIO")]
  pub stdio: bool,
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
