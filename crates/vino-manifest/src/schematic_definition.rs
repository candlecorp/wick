use std::collections::HashMap;
use std::convert::{
  TryFrom,
  TryInto,
};
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
/// The SchematicDefinition struct is a normalized representation of a Vino [SchematicManifest].
/// It handles the job of translating manifest versions into a consistent data structure.
pub struct SchematicDefinition {
  /// The name of the schematic.
  pub name: String,
  /// A mapping of references to the components they refer to.
  pub components: HashMap<String, ComponentDefinition>,
  /// A list of connections from and to ports on references defined in the components field.
  pub connections: Vec<ConnectionDefinition>,
  /// A list of [ProviderDefinition]s with namespaces and initialization configuration.
  pub providers: Vec<ProviderDefinition>,
  /// Reserved
  pub constraints: HashMap<String, String>,
}

impl SchematicDefinition {
  /// Get the name as an owned [String]
  #[must_use]
  pub fn get_name(&self) -> String {
    self.name.clone()
  }
  /// Get a [ComponentDefinition] by reference
  #[must_use]
  pub fn get_component(&self, reference: &str) -> Option<ComponentDefinition> {
    self.components.get(reference).cloned()
  }
}

impl TryFrom<crate::v0::SchematicManifest> for SchematicDefinition {
  type Error = Error;

  fn try_from(manifest: crate::v0::SchematicManifest) -> Result<Self> {
    Ok(Self {
      name: manifest.name.clone(),
      components: manifest
        .components
        .clone()
        .into_iter()
        .map(|(key, val)| Ok((key, val.try_into()?)))
        .filter_map(Result::ok)
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
    })
  }
}

impl TryFrom<SchematicManifest> for SchematicDefinition {
  type Error = Error;

  fn try_from(manifest: SchematicManifest) -> Result<Self> {
    let def = match manifest {
      SchematicManifest::V0(manifest) => manifest.try_into()?,
    };
    Ok(def)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A definition of a component used to reference a component registered under a provider.
/// Note: [ComponentDefinition] include embed the concept of a namespace so two identical
/// components registered on different namespaces will not be equal.
pub struct ComponentDefinition {
  /// The component's name
  pub name: String,
  /// The namespace the component was registered under
  pub namespace: String,
  /// The fully qualified ID used to reference the component.
  pub id: String,
  /// Reserved
  pub metadata: Option<String>,
}

impl ComponentDefinition {
  /// Quick way to create a [ComponentDefinition] from code. Used mostly in testing.
  #[must_use]
  pub fn new(namespace: &str, name: &str) -> Self {
    Self {
      name: name.to_owned(),
      namespace: namespace.to_owned(),
      id: format!("{}::{}", namespace, name),
      metadata: None,
    }
  }
}

/// Parse a fully qualified component ID into its namespace & name parts
pub fn parse_id(id: &str) -> Result<(&str, &str)> {
  if !id.contains("::") {
    Err(Error::ComponentIdError(id.to_owned()))
  } else {
    id.split_once("::")
      .map(|(ns, name)| Ok((ns, name)))
      .unwrap()
  }
}

impl ComponentDefinition {
  /// Parse a fully qualified component ID into its namespace & name parts
  pub fn parse_id(&self) -> Result<(&str, &str)> {
    parse_id(&self.id)
  }
}

impl TryFrom<crate::v0::ComponentDefinition> for ComponentDefinition {
  type Error = Error;
  fn try_from(def: crate::v0::ComponentDefinition) -> Result<Self> {
    let (ns, name) = parse_id(&def.id)?;
    Ok(ComponentDefinition {
      id: def.id.clone(),
      namespace: ns.to_owned(),
      name: name.to_owned(),
      metadata: None,
    })
  }
}

impl TryFrom<&crate::v0::ComponentDefinition> for ComponentDefinition {
  type Error = Error;
  fn try_from(def: &crate::v0::ComponentDefinition) -> Result<Self> {
    let (ns, name) = parse_id(&def.id)?;
    Ok(ComponentDefinition {
      id: def.id.clone(),
      namespace: ns.to_owned(),
      name: name.to_owned(),
      metadata: None,
    })
  }
}

#[derive(Debug, Clone)]
/// A definition of a Vino Provider with its namespace, how to retrieve or access it and its configuration.
pub struct ProviderDefinition {
  /// The namespace to reference the provider's components on
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kind of provider.
pub enum ProviderKind {
  /// Native providers included at compile-time in a Vino host
  Native = 0,
  /// The URL for a separately managed GRPC endpoint
  GrpcUrl = 1,
  /// A WaPC WebAssembly provider
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

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A [ConnectionDefinition] defines the link between an upstream and downstream port as well as
/// the default value to use in the case of an exception.
pub struct ConnectionDefinition {
  /// The upstream [ConnectionTargetDefinition] (port)
  pub from: ConnectionTargetDefinition,
  /// The downstream [ConnectionTargetDefinition] (port)
  pub to: ConnectionTargetDefinition,
  /// The default data to use in the case of an Error, represented as a JSON string.
  pub default: Option<String>,
}

impl ConnectionDefinition {
  /// Format a list of [ConnectionDefinition]s into a String
  #[must_use]
  pub fn print_all(list: &[Self]) -> String {
    list
      .iter()
      .map(|c| c.to_string())
      .collect::<Vec<String>>()
      .join(", ")
  }
}

impl From<crate::v0::ConnectionDefinition> for ConnectionDefinition {
  fn from(def: crate::v0::ConnectionDefinition) -> Self {
    ConnectionDefinition {
      from: def.from.into(),
      to: def.to.into(),
      default: def.default,
    }
  }
}

impl Display for ConnectionDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} => {}", self.from, self.to)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]

/// A [ConnectionTargetDefinition] is the link to a port for a specific reference of a component.
pub struct ConnectionTargetDefinition {
  /// A schematic-wide unique identifier for a [ComponentDefinition]
  pub reference: String,
  /// A port on the referenced [ComponentDefinition]
  pub port: String,
}

impl ConnectionTargetDefinition {
  /// Constructor for a [ConnectionTargetDefinition]. Used mostly in test code.
  pub fn new<T: AsRef<str>, U: AsRef<str>>(reference: T, port: U) -> Self {
    Self {
      reference: reference.as_ref().to_owned(),
      port: port.as_ref().to_owned(),
    }
  }
}

impl<T, U> From<(T, U)> for ConnectionTargetDefinition
where
  T: Display,
  U: Display,
{
  fn from((reference, port): (T, U)) -> Self {
    ConnectionTargetDefinition {
      reference: reference.to_string(),
      port: port.to_string(),
    }
  }
}

impl Display for ConnectionTargetDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}[{}]", self.reference, self.port)
  }
}

impl From<crate::v0::ConnectionTargetDefinition> for ConnectionTargetDefinition {
  fn from(def: crate::v0::ConnectionTargetDefinition) -> Self {
    ConnectionTargetDefinition {
      reference: def.reference,
      port: def.port,
    }
  }
}
