use std::net::Ipv4Addr;
use std::time::Duration;

use crate::config;

#[derive(Debug, Clone, Default)]
#[must_use]
/// Configuration options for the host to use at startup.
pub struct HostConfig {
  /// Flag to allow/disallow `:latest` tags for OCI artifacts.
  pub allow_latest: bool,

  /// The list of registries to connect via HTTP rather than HTTPS.
  pub insecure_registries: Vec<String>,

  /// The timeout for network requests.
  pub timeout: Duration,

  /// Configuration for the GRPC server.
  pub rpc: Option<HttpConfig>,
}

#[derive(Debug, Default, Clone)]
/// Configuration for HTTP/S servers.
pub struct HttpConfig {
  /// Enable/disable the server.
  pub enabled: bool,

  /// The port to bind to.
  pub port: Option<u16>,

  /// The address to bind to.
  pub address: Option<Ipv4Addr>,

  /// Path to pem file for TLS.
  pub pem: Option<config::AssetReference>,

  /// Path to key file for TLS.
  pub key: Option<config::AssetReference>,

  /// Path to CA file.
  pub ca: Option<config::AssetReference>,
}
