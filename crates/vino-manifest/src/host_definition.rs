use std::convert::{
  TryFrom,
  TryInto,
};
use std::net::Ipv4Addr;
use std::path::{
  Path,
  PathBuf,
};
use std::str::FromStr;
use std::time::Duration;

use crate::error::ManifestError;
use crate::{
  HostManifest,
  Loadable,
  NetworkDefinition,
  Result,
};

#[derive(Debug, Clone, Default)]
/// The [HostDefinition] struct is a normalized representation of a Vino [HostManifest].
/// It handles the job of translating manifest versions into a consistent data structure.
pub struct HostDefinition {
  /// The [NetworkDefinition] from the manifest.
  pub network: NetworkDefinition,

  /// The default schematic to execute via `vino run`.
  pub default_schematic: String,

  /// Configuration options.
  pub host: HostConfig,
}

impl TryFrom<HostManifest> for HostDefinition {
  type Error = ManifestError;

  fn try_from(manifest: HostManifest) -> Result<Self> {
    let result = match manifest {
      HostManifest::V0(manifest) => Self {
        host: manifest.host.clone().try_into()?,
        default_schematic: manifest.default_schematic.clone(),
        network: manifest.network.into(),
      },
    };
    Ok(result)
  }
}

impl HostDefinition {
  /// Utility function to automate loading a manifest from a file.
  pub fn load_from_file(path: &Path) -> Result<HostDefinition> {
    let manifest = crate::HostManifest::load_from_file(path)?;
    HostDefinition::try_from(manifest)
  }
}

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

  /// Configuration for the Lattice.
  pub lattice: Option<LatticeConfig>,

  /// Configuration for the GRPC server.
  pub rpc: Option<HttpConfig>,

  /// Configuration for the development HTTP 1 server.
  pub http: Option<HttpConfig>,
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
/// Configuration used to connect to the lattice.
pub struct LatticeConfig {
  /// Enable/disable the lattice connection.
  pub enabled: bool,
  /// The address of the NATS server.
  pub address: String,
  /// The path to the NATS credsfile.
  pub creds_path: Option<PathBuf>,
  /// The NATS token.
  pub token: Option<String>,
}

impl TryFrom<crate::v0::HostConfig> for HostConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v0::HostConfig) -> Result<Self> {
    Ok(Self {
      allow_latest: def.allow_latest,
      insecure_registries: def.insecure_registries,
      timeout: Duration::from_millis(def.timeout),
      id: def.id,
      lattice: def.lattice.and_then(|v| v.try_into().ok()),
      rpc: def.rpc.and_then(|v| v.try_into().ok()),
      http: def.http.and_then(|v| v.try_into().ok()),
    })
  }
}

impl TryFrom<crate::v0::LatticeConfig> for LatticeConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v0::LatticeConfig) -> Result<Self> {
    Ok(Self {
      enabled: def.enabled,
      address: def.address,
      creds_path: opt_str_to_pathbuf(&def.creds_path)?,
      token: def.token,
    })
  }
}

impl TryFrom<crate::v0::HttpConfig> for HttpConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v0::HttpConfig) -> Result<Self> {
    Ok(Self {
      enabled: def.enabled,
      port: def.port,
      address: opt_str_to_ipv4addr(&def.address)?,
      pem: opt_str_to_pathbuf(&def.pem)?,
      key: opt_str_to_pathbuf(&def.key)?,
      ca: opt_str_to_pathbuf(&def.ca)?,
    })
  }
}

fn opt_str_to_pathbuf(v: &Option<String>) -> Result<Option<PathBuf>> {
  Ok(match v {
    Some(v) => Some(PathBuf::from_str(v).map_err(|e| ManifestError::BadPath(e.to_string()))?),
    None => None,
  })
}

fn opt_str_to_ipv4addr(v: &Option<String>) -> Result<Option<Ipv4Addr>> {
  Ok(match v {
    Some(v) => Some(Ipv4Addr::from_str(v).map_err(|e| ManifestError::BadIpAddress(e.to_string()))?),
    None => None,
  })
}
