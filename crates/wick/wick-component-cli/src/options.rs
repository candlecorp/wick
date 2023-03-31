use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::time::Duration;

use clap::Args;
use logger::LoggingOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
/// Server configuration options.
pub struct Options {
  /// RPC server options.
  pub rpc: Option<ServerOptions>,
  /// The ID of the server.
  pub id: String,
  /// The timeout for network requests.
  pub timeout: Duration,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      id: uuid::Uuid::new_v4().as_hyphenated().to_string(),
      rpc: Default::default(),
      timeout: Default::default(),
    }
  }
}

#[derive(Debug, Default, Clone)]
/// Configuration used to connect to the mesh
pub struct MeshOptions {
  /// Enable/disable the mesh connection.
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
  #[cfg(feature = "grpc")]
  pub pem: Option<wick_config::config::LocationReference>,

  /// Path to key file for TLS.
  #[cfg(feature = "grpc")]
  pub key: Option<wick_config::config::LocationReference>,

  /// Path to CA file.
  #[cfg(feature = "grpc")]
  pub ca: Option<wick_config::config::LocationReference>,
}

#[allow(clippy::expect_used)]
impl From<DefaultCliOptions> for Options {
  fn from(opts: DefaultCliOptions) -> Self {
    let rpc = Some(ServerOptions {
      enabled: opts.rpc_enabled,
      port: opts.rpc_port,
      address: opts.rpc_address,
      #[cfg(feature = "grpc")]
      pem: opts.rpc_pem.map(wick_config::config::LocationReference::new),
      #[cfg(feature = "grpc")]
      key: opts.rpc_key.map(wick_config::config::LocationReference::new),

      #[cfg(feature = "grpc")]
      ca: opts.rpc_ca.map(wick_config::config::LocationReference::new),
    });

    let id = opts
      .id
      .unwrap_or_else(|| uuid::Uuid::new_v4().as_hyphenated().to_string());

    Options {
      rpc,
      timeout: Duration::from_millis(opts.timeout.unwrap_or(5000)),
      id,
    }
  }
}

impl From<DefaultCliOptions> for LoggingOptions {
  fn from(opts: DefaultCliOptions) -> Self {
    opts.logging
  }
}

/// Names of the environment variables used for fallback values.
pub mod env {
  macro_rules! env_var {
    (  $x:ident  ) => {
      /// Environment variable fallback for CLI options
      pub const $x: &str = stringify!($x);
    };
  }

  env_var!(WICK_COLLECTION_ID);
  env_var!(WICK_TIMEOUT);

  env_var!(WICK_RPC_ENABLED);
  env_var!(WICK_RPC_PORT);
  env_var!(WICK_RPC_ADDRESS);
  env_var!(WICK_RPC_KEY);
  env_var!(WICK_RPC_PEM);
  env_var!(WICK_RPC_CA);

  env_var!(NATS_URL);
  env_var!(NATS_CREDSFILE);
  env_var!(NATS_TOKEN);
}

#[derive(Debug, Clone, Default, Args, Serialize, Deserialize)]
/// Command line options for s.
pub struct DefaultCliOptions {
  /// The unique ID of this client.
  #[clap(long = "id", env = env::WICK_COLLECTION_ID, action)]
  pub id: Option<String>,

  /// The timeout for outbound requests in ms.
  #[clap(long = "timeout", env = env::WICK_TIMEOUT, action)]
  pub timeout: Option<u64>,

  /// Logging options.
  #[clap(flatten)]
  pub logging: LoggingOptions,

  /// Enable the rpc server.
  #[clap(long = "rpc",  env = env::WICK_RPC_ENABLED, action)]
  pub rpc_enabled: bool,

  /// Port to listen on for GRPC server.
  #[clap(long = "rpc-port", env = env::WICK_RPC_PORT, action)]
  pub rpc_port: Option<u16>,

  /// IP address to bind to for GRPC server.
  #[clap(long = "rpc-address", env = env::WICK_RPC_ADDRESS, action)]
  pub rpc_address: Option<Ipv4Addr>,

  /// Path to pem file for TLS for GRPC server.
  #[clap(long = "rpc-pem", env = env::WICK_RPC_PEM, action)]
  pub rpc_pem: Option<String>,

  /// Path to key file for TLS for GRPC server.
  #[clap(long = "rpc-key", env = env::WICK_RPC_KEY, action)]
  pub rpc_key: Option<String>,

  /// Path to certificate authority for GRPC server.
  #[clap(long = "rpc-ca", env = env::WICK_RPC_CA, action)]
  pub rpc_ca: Option<String>,
}
