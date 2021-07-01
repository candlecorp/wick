use std::collections::HashMap;
use std::fmt::Display;

use serde::{
  Deserialize,
  Serialize,
};

use crate::{
  Error,
  Result,
  SchematicManifest,
};

#[derive(Debug, Clone, Default)]
pub struct SchematicDefinition {
  pub name: String,
  pub external: Vec<ExternalComponentDefinition>,
  pub components: HashMap<String, ComponentDefinition>,
  pub connections: Vec<ConnectionDefinition>,
  pub providers: Vec<ProviderDefinition>,
  pub constraints: HashMap<String, String>,
}

impl SchematicDefinition {
  pub fn from_manifest(manifest: &SchematicManifest) -> Self {
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
  pub fn get_name(&self) -> String {
    self.name.clone()
  }
  pub fn get_component(&self, reference: &str) -> Option<ComponentDefinition> {
    self.components.get(reference).cloned()
  }
}

impl From<crate::v0::SchematicManifest> for SchematicDefinition {
  fn from(def: crate::v0::SchematicManifest) -> Self {
    Self::from_manifest(&crate::SchematicManifest::V0(def))
  }
}

#[derive(Debug, Clone)]
pub struct ExternalComponentDefinition {
  pub alias: Option<String>,
  pub reference: String,
  pub key: String,
}

impl From<crate::v0::ExternalComponentDefinition> for ExternalComponentDefinition {
  fn from(def: crate::v0::ExternalComponentDefinition) -> Self {
    Self {
      alias: def.alias,
      key: def.key,
      reference: def.reference,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ComponentDefinition {
  /// Reserved
  pub metadata: Option<String>,
  /// The ID used to reference the component. Can be a public key or fully qualified namespace reference
  pub id: String,
}

pub fn parse_namespace(id: &str) -> Result<(String, String)> {
  if !id.contains("::") {
    Err(Error::ComponentIdError(id.to_string()))
  } else {
    id.split_once("::")
      .map(|(ns, name)| Ok((ns.to_string(), name.to_string())))
      .unwrap()
  }
}

impl ComponentDefinition {
  pub fn parse_namespace(&self) -> Result<(String, String)> {
    parse_namespace(&self.id)
  }
}

impl From<crate::v0::ComponentDefinition> for ComponentDefinition {
  fn from(def: crate::v0::ComponentDefinition) -> Self {
    ComponentDefinition {
      id: def.id,
      metadata: None,
    }
  }
}

impl From<&crate::v0::ComponentDefinition> for ComponentDefinition {
  fn from(def: &crate::v0::ComponentDefinition) -> Self {
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

impl From<crate::v0::ProviderDefinition> for ProviderDefinition {
  fn from(def: crate::v0::ProviderDefinition) -> Self {
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

  Wapc = 2,
}

impl From<crate::v0::ProviderKind> for ProviderKind {
  fn from(def: crate::v0::ProviderKind) -> Self {
    match def {
      crate::v0::ProviderKind::Native => ProviderKind::Native,
      crate::v0::ProviderKind::GrpcUrl => ProviderKind::GrpcUrl,
      crate::v0::ProviderKind::WaPC => ProviderKind::Wapc,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ConnectionDefinition {
  pub from: ConnectionTargetDefinition,
  pub to: ConnectionTargetDefinition,
}

impl From<crate::v0::ConnectionDefinition> for ConnectionDefinition {
  fn from(def: crate::v0::ConnectionDefinition) -> Self {
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

impl From<crate::v0::ConnectionTargetDefinition> for ConnectionTargetDefinition {
  fn from(def: crate::v0::ConnectionTargetDefinition) -> Self {
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
