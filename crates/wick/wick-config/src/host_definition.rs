use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::time::Duration;

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

  /// The host ID.
  pub id: Option<String>,

  /// Configuration for the Mesh.
  pub mesh: Option<MeshConfig>,

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
  pub pem: Option<PathBuf>,

  /// Path to key file for TLS.
  pub key: Option<PathBuf>,

  /// Path to CA file.
  pub ca: Option<PathBuf>,
}

#[derive(Debug, Default, Clone)]
/// Configuration used to connect to the mesh.
pub struct MeshConfig {
  /// Enable/disable the mesh connection.
  pub enabled: bool,

  /// The address of the NATS server.
  pub address: String,

  /// The path to the NATS credsfile.
  pub creds_path: Option<PathBuf>,

  /// The NATS token.
  pub token: Option<String>,
}
