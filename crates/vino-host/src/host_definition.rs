use std::path::Path;

use vino_manifest::{
  HostManifest,
  Loadable,
};
use vino_runtime::NetworkDefinition;

use crate::Result;

#[derive(Debug, Clone, Default)]
pub struct HostDefinition {
  pub network: NetworkDefinition,

  pub default_schematic: String,

  pub config: CommonConfiguration,
}

impl HostDefinition {
  pub(crate) fn new(manifest: &HostManifest) -> Self {
    match manifest {
      HostManifest::V0(manifest) => Self {
        config: manifest.nats.clone().into(),
        default_schematic: manifest.default_schematic.clone(),
        network: manifest.network.clone().into(),
      },
    }
  }
  pub fn load_from_file(path: &Path) -> Result<HostDefinition> {
    let manifest = vino_manifest::HostManifest::load_from_file(path)?;
    Ok(HostDefinition::new(&manifest))
  }
}

#[derive(Debug, Clone, Default)]
pub struct CommonConfiguration {
  pub rpc_host: String,
  pub rpc_port: String,
  pub rpc_credsfile: Option<String>,
  pub rpc_jwt: Option<String>,
  pub rpc_seed: Option<String>,
  pub control_host: String,
  pub control_port: String,
  pub control_credsfile: Option<String>,
  pub control_jwt: Option<String>,
  pub control_seed: Option<String>,
  pub allow_oci_latest: bool,
  pub allowed_insecure: Vec<String>,
}

impl CommonConfiguration {
  pub fn new(manifest: &vino_manifest::v0::NatsConfiguration) -> Self {
    Self {
      rpc_host: manifest.rpc_host.clone(),
      rpc_port: manifest.rpc_port.clone(),
      rpc_credsfile: manifest.rpc_credsfile.clone(),
      rpc_jwt: manifest.rpc_jwt.clone(),
      rpc_seed: manifest.rpc_seed.clone(),
      control_host: manifest.control_host.clone(),
      control_port: manifest.control_port.clone(),
      control_credsfile: manifest.control_credsfile.clone(),
      control_jwt: manifest.control_jwt.clone(),
      control_seed: manifest.control_seed.clone(),
      allow_oci_latest: manifest.allow_oci_latest,
      allowed_insecure: manifest.allowed_insecure.clone(),
    }
  }
}

impl From<vino_manifest::v0::NatsConfiguration> for CommonConfiguration {
  fn from(def: vino_manifest::v0::NatsConfiguration) -> Self {
    Self::new(&def)
  }
}
