use std::collections::HashMap;
use std::fmt::Display;

use serde::{
  Deserialize,
  Serialize,
};
use vino_manifest::SchematicManifest;

use crate::{
  Error,
  Result,
};

#[derive(Debug, Clone, Default)]
pub struct SchematicDefinition {
  pub name: String,
  pub(crate) external: Vec<ExternalComponentDefinition>,
  pub(crate) components: HashMap<String, ComponentDefinition>,
  pub(crate) connections: Vec<ConnectionDefinition>,
  pub(crate) providers: Vec<ProviderDefinition>,
  pub(crate) constraints: HashMap<String, String>,
}

impl SchematicDefinition {
  pub(crate) fn new(manifest: &SchematicManifest) -> Self {
    match manifest {
      SchematicManifest::V0(manifest) => Self {
        name: manifest.name.clone(),
        components: manifest
          .components
          .clone()
          .into_iter()
          .map(|(key, val)| (key, val.into()))
          .collect(),
        connections: manifest
          .connections
          .clone()
          .into_iter()
          .map(|def| def.into())
          .collect(),
        providers: manifest
          .providers
          .clone()
          .into_iter()
          .map(|def| def.into())
          .collect(),
        constraints: manifest.constraints.clone().into_iter().collect(),
        external: manifest
          .external
          .clone()
          .into_iter()
          .map(|def| def.into())
          .collect(),
      },
    }
  }
  pub(crate) fn get_name(&self) -> String {
    self.name.clone()
  }
  pub(crate) fn get_component(&self, name: &str) -> Option<&ComponentDefinition> {
    self.components.get(name)
  }

  pub(crate) fn get_output_names(&self) -> Vec<String> {
    self
      .connections
      .iter()
      .filter(|conn| conn.to.instance == crate::SCHEMATIC_OUTPUT)
      .map(|conn| conn.to.port.to_string())
      .collect()
  }
  pub(crate) fn id_to_ref(&self, id: &str) -> Result<String> {
    if id.starts_with(crate::VINO_NAMESPACE) {
      Ok(id.to_string())
    } else {
      for component in &self.external {
        if id == component.key || Some(id.to_string()) == component.alias {
          return Ok(component.reference.to_string());
        }
      }
      Err(Error::SchematicError(format!(
        "No external component found with alias or key {}",
        id
      )))
    }
  }
}

impl From<vino_manifest::v0::SchematicManifest> for SchematicDefinition {
  fn from(def: vino_manifest::v0::SchematicManifest) -> Self {
    Self::new(&vino_manifest::SchematicManifest::V0(def))
  }
}

#[derive(Debug, Clone)]
pub struct ExternalComponentDefinition {
  pub alias: Option<String>,
  pub reference: String,
  pub key: String,
}

impl From<vino_manifest::v0::ExternalComponentDefinition> for ExternalComponentDefinition {
  fn from(def: vino_manifest::v0::ExternalComponentDefinition) -> Self {
    Self {
      alias: def.alias,
      key: def.key,
      reference: def.reference,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ComponentDefinition {
  pub metadata: Option<String>,
  pub id: String,
}

impl From<vino_manifest::v0::ComponentDefinition> for ComponentDefinition {
  fn from(def: vino_manifest::v0::ComponentDefinition) -> Self {
    ComponentDefinition {
      id: def.id,
      metadata: None,
    }
  }
}

impl From<&vino_manifest::v0::ComponentDefinition> for ComponentDefinition {
  fn from(def: &vino_manifest::v0::ComponentDefinition) -> Self {
    ComponentDefinition {
      id: def.id.to_string(),
      metadata: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ProviderDefinition {
  /// The namespace to reference the provider&#x27;s components on
  pub namespace: String,
  /// The kind/type of the provider
  pub kind: ProviderKind,
  /// The reference/location of the provider
  pub reference: String,
  /// Data or configuration to pass to the provider initialization
  pub data: HashMap<String, String>,
}

impl From<vino_manifest::v0::ProviderDefinition> for ProviderDefinition {
  fn from(def: vino_manifest::v0::ProviderDefinition) -> Self {
    ProviderDefinition {
      namespace: def.namespace,
      kind: def.kind.into(),
      reference: def.reference,
      data: def.data,
    }
  }
}

#[derive(Debug, Clone)]
/// Kind of provider,
pub enum ProviderKind {
  /// Native providers included at compile-time in a Vino host
  Native = 0,
  /// The URL for a separately managed GRPC endpoint
  GrpcUrl = 1,
}

impl From<vino_manifest::v0::ProviderKind> for ProviderKind {
  fn from(def: vino_manifest::v0::ProviderKind) -> Self {
    match def {
      vino_manifest::v0::ProviderKind::Native => ProviderKind::Native,
      vino_manifest::v0::ProviderKind::GrpcUrl => ProviderKind::GrpcUrl,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ConnectionDefinition {
  pub from: ConnectionTargetDefinition,
  pub to: ConnectionTargetDefinition,
}

impl From<vino_manifest::v0::ConnectionDefinition> for ConnectionDefinition {
  fn from(def: vino_manifest::v0::ConnectionDefinition) -> Self {
    ConnectionDefinition {
      from: def.from.into(),
      to: def.to.into(),
    }
  }
}

impl Display for ConnectionDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} => {}", self.from, self.to)
  }
}

impl Display for ConnectionTargetDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}[{}]", self.instance, self.port)
  }
}

impl From<vino_manifest::v0::ConnectionTargetDefinition> for ConnectionTargetDefinition {
  fn from(def: vino_manifest::v0::ConnectionTargetDefinition) -> Self {
    ConnectionTargetDefinition {
      instance: def.instance,
      port: def.port,
    }
  }
}

impl ConnectionDefinition {
  pub fn print_all(list: &[Self]) -> String {
    list
      .iter()
      .map(|c| c.to_string())
      .collect::<Vec<String>>()
      .join(", ")
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTargetDefinition {
  pub instance: String,
  pub port: String,
}

impl<T, U> From<(T, U)> for ConnectionTargetDefinition
where
  T: Display,
  U: Display,
{
  fn from((instance, port): (T, U)) -> Self {
    ConnectionTargetDefinition {
      instance: instance.to_string(),
      port: port.to_string(),
    }
  }
}
