use std::convert::TryInto;

use serde_json::Value;

use crate::schematic_definition::SchematicDefinition;
use crate::{Error, NetworkManifest};

#[derive(Debug, Default, Clone)]

/// The NetworkDefinition struct is a normalized representation of a Vino [NetworkManifest].
/// It handles the job of translating manifest versions into a consistent data structure.
#[must_use]
pub struct NetworkDefinition {
  /// The name of the Network if provided.
  pub name: Option<String>,
  /// An optional entrypoint for the network.
  pub entry: Option<EntrypointDefinition>,
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
      entry: def.entry.clone().map(EntrypointDefinition::from),
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

#[derive(Debug, Clone)]
/// A definition of a Vino Provider with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct EntrypointDefinition {
  /// The reference/location of the provider.
  pub reference: String,
  /// Data or configuration to pass to the provider initialization.
  pub data: Value,
}

impl From<crate::v0::EntrypointDefinition> for EntrypointDefinition {
  fn from(def: crate::v0::EntrypointDefinition) -> Self {
    EntrypointDefinition {
      reference: def.reference,
      data: def.data,
    }
  }
}

#[derive(Debug, Clone)]
/// A definition of a Vino Provider with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct ProviderDefinition {
  /// The namespace to reference the provider's components on.
  pub namespace: String,
  /// The kind/type of the provider.
  pub kind: ProviderKind,
  /// The reference/location of the provider.
  pub reference: String,
  /// Data or configuration to pass to the provider initialization.
  pub data: Value,
}

impl From<&crate::v0::ProviderDefinition> for ProviderDefinition {
  fn from(def: &crate::v0::ProviderDefinition) -> Self {
    ProviderDefinition {
      namespace: def.namespace.clone(),
      kind: def.kind.into(),
      reference: def.reference.clone(),
      data: def.data.clone(),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kind of provider.
pub enum ProviderKind {
  /// Native providers included at compile-time in a Vino host.
  Native = 0,
  /// The URL for a separately managed GRPC endpoint.
  GrpcUrl = 1,
  /// A WaPC WebAssembly provider.
  Wapc = 2,
  /// A provider accessible via a connected lattice.
  Lattice = 3,
  /// A local or remote Network definition.
  Network = 4,
  /// A GRPC provider binary.
  Par = 5,
}

impl From<crate::v0::ProviderKind> for ProviderKind {
  fn from(def: crate::v0::ProviderKind) -> Self {
    match def {
      crate::v0::ProviderKind::Native => ProviderKind::Native,
      crate::v0::ProviderKind::Par => ProviderKind::Par,
      crate::v0::ProviderKind::GrpcUrl => ProviderKind::GrpcUrl,
      crate::v0::ProviderKind::WaPC => ProviderKind::Wapc,
      crate::v0::ProviderKind::Lattice => ProviderKind::Lattice,
      crate::v0::ProviderKind::Network => ProviderKind::Network,
    }
  }
}
