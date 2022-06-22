use std::convert::TryInto;

use serde_json::Value;

use crate::flow_definition::Flow;
use crate::Error;

#[derive(Debug, Default, Clone)]

/// The NetworkDefinition struct is a normalized representation of a Wasmflow [NetworkManifest].
/// It handles the job of translating manifest versions into a consistent data structure.
#[must_use]
pub struct NetworkDefinition {
  /// The name of the Network if provided.
  pub name: Option<String>,
  /// An optional entrypoint for the network.
  pub triggers: Option<EntrypointDefinition>,
  /// A list of SchematicDefinitions.
  pub schematics: Vec<Flow>,
  /// A list of CollectionDefinitions.
  pub collections: Vec<CollectionDefinition>,
}

impl NetworkDefinition {
  /// Get a [SchematicDefinition] by name.
  #[must_use]
  pub fn schematic(&self, name: &str) -> Option<&Flow> {
    self.schematics.iter().find(|s| s.name == name)
  }
}

impl TryFrom<&crate::v0::NetworkManifest> for NetworkDefinition {
  type Error = Error;
  fn try_from(def: &crate::v0::NetworkManifest) -> Result<Self, Error> {
    let schematics: Result<Vec<Flow>, Error> = def.schematics.iter().map(|val| val.try_into()).collect();
    let collections = def.collections.iter().map(|val| val.into()).collect();
    Ok(Self {
      name: def.name.clone(),
      schematics: schematics?,
      triggers: def.triggers.clone().map(EntrypointDefinition::from),
      collections,
    })
  }
}

#[derive(Debug, Clone)]
/// A definition of a Wasmflow Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct EntrypointDefinition {
  /// The reference/location of the collection.
  pub reference: String,
  /// Data or configuration to pass to the collection initialization.
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

impl From<crate::v1::EntrypointDefinition> for EntrypointDefinition {
  fn from(def: crate::v1::EntrypointDefinition) -> Self {
    EntrypointDefinition {
      reference: def.reference,
      data: def.config,
    }
  }
}

#[derive(Debug, Clone)]
/// A definition of a Wasmflow Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct CollectionDefinition {
  /// The namespace to reference the collection's components on.
  pub namespace: String,
  /// The kind/type of the collection.
  pub kind: CollectionKind,
  /// The reference/location of the collection.
  pub reference: String,
  /// Data or configuration to pass to the collection initialization.
  pub data: Value,
}

impl From<&crate::v0::CollectionDefinition> for CollectionDefinition {
  fn from(def: &crate::v0::CollectionDefinition) -> Self {
    CollectionDefinition {
      namespace: def.namespace.clone(),
      kind: def.kind.into(),
      reference: def.reference.clone(),
      data: def.data.clone(),
    }
  }
}

impl From<(String, crate::v1::CollectionDefinition)> for CollectionDefinition {
  fn from(def: (String, crate::v1::CollectionDefinition)) -> Self {
    CollectionDefinition {
      namespace: def.0,
      kind: def.1.kind.into(),
      reference: def.1.reference,
      data: def.1.config,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kind of collection.
pub enum CollectionKind {
  /// Native collections included at compile-time in a Wasmflow host.
  Native = 0,
  /// The URL for a separately managed GRPC endpoint.
  GrpcUrl = 1,
  /// A WaPC WebAssembly collection.
  Wasm = 2,
  /// A collection accessible via a connected mesh.
  Mesh = 3,
  /// A local or remote Network definition.
  Network = 4,
  /// A GRPC collection binary.
  Par = 5,
}

impl From<crate::v0::CollectionKind> for CollectionKind {
  fn from(def: crate::v0::CollectionKind) -> Self {
    match def {
      crate::v0::CollectionKind::Native => CollectionKind::Native,
      crate::v0::CollectionKind::Par => CollectionKind::Par,
      crate::v0::CollectionKind::GrpcUrl => CollectionKind::GrpcUrl,
      crate::v0::CollectionKind::WaPC => CollectionKind::Wasm,
      crate::v0::CollectionKind::Mesh => CollectionKind::Mesh,
      crate::v0::CollectionKind::Network => CollectionKind::Network,
    }
  }
}

impl From<crate::v1::CollectionKind> for CollectionKind {
  fn from(def: crate::v1::CollectionKind) -> Self {
    match def {
      crate::v1::CollectionKind::Native => CollectionKind::Native,
      crate::v1::CollectionKind::Par => CollectionKind::Par,
      crate::v1::CollectionKind::GrpcUrl => CollectionKind::GrpcUrl,
      crate::v1::CollectionKind::WASM => CollectionKind::Wasm,
      crate::v1::CollectionKind::Mesh => CollectionKind::Mesh,
      crate::v1::CollectionKind::Network => CollectionKind::Network,
    }
  }
}
