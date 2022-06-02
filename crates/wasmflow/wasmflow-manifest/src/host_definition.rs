use std::convert::{TryFrom, TryInto};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use crate::error::ManifestError;
use crate::v0::HOST_CONFIG_TIMEOUT;
use crate::{HostManifest, Loadable, NetworkDefinition, Result};

#[derive(Debug, Clone, Default)]
/// The [HostDefinition] struct is a normalized representation of a Wasmflow [HostManifest].
/// It handles the job of translating manifest versions into a consistent data structure.
pub struct HostDefinition {
  /// The location where the HostDefinition was loaded from, if known.
  pub source: Option<String>,

  /// The [NetworkDefinition] from the manifest.
  pub network: NetworkDefinition,

  /// The default schematic to execute if none provided.
  pub default_schematic: Option<String>,

  /// Configuration options.
  pub host: HostConfig,
}

impl TryFrom<HostManifest> for HostDefinition {
  type Error = ManifestError;

  fn try_from(manifest: HostManifest) -> Result<Self> {
    let result = match manifest {
      HostManifest::V0(manifest) => {
        // Hack. This is due to serde/serde_yaml using Default::default() implementations
        // even if the inner struct is filled with #[serde(default="...")] attributes.
        // See: https://github.com/serde-rs/serde/issues/1416
        // See: https://github.com/dtolnay/request-for-implementation/issues/4
        // See: https://github.com/TedDriggs/serde_default (unpublished and out of date)
        let mut host_config: HostConfig = manifest.host.clone().try_into()?;
        if host_config.timeout == Duration::from_millis(0) {
          host_config.timeout = Duration::from_millis(HOST_CONFIG_TIMEOUT());
        }
        // End hack.
        Self {
          source: None,
          host: host_config,
          default_schematic: manifest.default_schematic.clone(),
          network: (&manifest.network).try_into()?,
        }
      }
    };
    Ok(result)
  }
}

impl HostDefinition {
  /// Utility function to automate loading a manifest from a file.
  pub fn load_from_file(path: impl AsRef<Path>) -> Result<HostDefinition> {
    let manifest = crate::HostManifest::load_from_file(path.as_ref())?;

    let mut def = HostDefinition::try_from(manifest)?;
    def.source = Some(path.as_ref().to_string_lossy().to_string());
    Ok(def)
  }

  /// Utility function to automate loading a manifest from a byte array.
  pub fn load_from_bytes(source: Option<String>, src: &[u8]) -> Result<HostDefinition> {
    let manifest = crate::HostManifest::load_from_bytes(src)?;
    let mut def = HostDefinition::try_from(manifest)?;
    def.source = source;
    Ok(def)
  }

  /// Get the inner [NetworkDefinition].
  pub fn network(&self) -> &NetworkDefinition {
    &self.network
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
