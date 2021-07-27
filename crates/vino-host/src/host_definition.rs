use std::path::Path;

use vino_manifest::{
  HostManifest,
  Loadable,
  NetworkDefinition,
};

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
        config: manifest.config.clone().into(),
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
  pub allow_latest: bool,
  pub insecure_registries: Vec<String>,
}

impl CommonConfiguration {
  #[must_use]
  pub fn new(manifest: vino_manifest::v0::HostConfig) -> Self {
    Self {
      allow_latest: manifest.allow_latest,
      insecure_registries: manifest.insecure_registries,
    }
  }
}

impl From<vino_manifest::v0::HostConfig> for CommonConfiguration {
  fn from(def: vino_manifest::v0::HostConfig) -> Self {
    Self::new(def)
  }
}
