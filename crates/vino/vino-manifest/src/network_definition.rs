use std::convert::TryInto;

use crate::schematic_definition::SchematicDefinition;
use crate::{Error, NetworkManifest, ProviderDefinition};

#[derive(Debug, Default, Clone)]

/// The NetworkDefinition struct is a normalized representation of a Vino [NetworkManifest].
/// It handles the job of translating manifest versions into a consistent data structure.
#[must_use]
pub struct NetworkDefinition {
  /// The name of the Network if provided.
  pub name: Option<String>,
  /// A list of SchematicDefinitions.
  pub schematics: Vec<SchematicDefinition>,
  /// A list of ProviderDefinitions.
  pub providers: Vec<ProviderDefinition>,
}

impl NetworkDefinition {
  /// Get a [SchematicDefinition] by name.
  #[must_use]
  pub fn schematic(&self, name: &str) -> Option<&SchematicDefinition> {
    self.schematics.iter().find(|s| s.name == name)
  }
}

impl TryFrom<&crate::v0::NetworkManifest> for NetworkDefinition {
  type Error = Error;
  fn try_from(def: &crate::v0::NetworkManifest) -> Result<Self, Error> {
    let schematics: Result<Vec<SchematicDefinition>, Error> = def.schematics.iter().map(|val| val.try_into()).collect();
    let providers = def.providers.iter().map(|val| val.into()).collect();
    Ok(Self {
      name: def.name.clone(),
      schematics: schematics?,
      providers,
    })
  }
}

impl TryFrom<NetworkManifest<'_>> for NetworkDefinition {
  type Error = Error;
  fn try_from(manifest: NetworkManifest) -> Result<Self, Error> {
    match manifest {
      NetworkManifest::V0(manifest) => manifest.try_into(),
    }
  }
}
