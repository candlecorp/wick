use std::convert::TryInto;

use crate::schematic_definition::SchematicDefinition;
use crate::NetworkManifest;

#[derive(Debug, Clone)]

/// The NetworkDefinition struct is a normalized representation of a Vino [NetworkManifest].
/// It handles the job of translating manifest versions into a consistent data structure.
pub struct NetworkDefinition {
  /// A list of SchematicDefinitions
  pub schematics: Vec<SchematicDefinition>,
}

impl NetworkDefinition {}

impl From<crate::v0::NetworkManifest> for NetworkDefinition {
  fn from(def: crate::v0::NetworkManifest) -> Self {
    Self {
      schematics: def
        .schematics
        .into_iter()
        .map(|val| val.try_into())
        .filter_map(Result::ok)
        .collect(),
    }
  }
}

impl From<NetworkManifest> for NetworkDefinition {
  fn from(manifest: NetworkManifest) -> Self {
    match manifest {
      NetworkManifest::V0(manifest) => Self {
        schematics: manifest
          .schematics
          .into_iter()
          .map(|val| val.try_into())
          .filter_map(Result::ok)
          .collect(),
      },
    }
  }
}

impl Default for NetworkDefinition {
  fn default() -> Self {
    Self { schematics: vec![] }
  }
}
