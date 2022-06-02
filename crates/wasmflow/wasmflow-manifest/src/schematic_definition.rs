use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasmflow_transport::MessageTransport;

use crate::default::{parse_default, process_default};
use crate::parse::parse_id;
use crate::{Error, Result, SchematicManifest};

#[derive(Debug, Clone, Default)]
/// The SchematicDefinition struct is a normalized representation of a Wasmflow [SchematicManifest].
/// It handles the job of translating manifest versions into a consistent data structure.
#[must_use]
pub struct SchematicDefinition {
  /// The name of the schematic.
  pub name: String,
  /// A mapping of instance names to the components they refer to.
  pub instances: HashMap<String, ComponentDefinition>,
  /// A list of connections from and to ports on instances defined in the instance map.
  pub connections: Vec<ConnectionDefinition>,
  /// A list of provider namespaces to expose to this schematic.
  pub providers: Vec<String>,
  /// Reserved.
  pub constraints: HashMap<String, String>,
}

impl SchematicDefinition {
  /// Get the name as an owned [String].
  #[must_use]
  pub fn get_name(&self) -> String {
    self.name.clone()
  }
  /// Get a [ComponentDefinition] by instance name.
  #[must_use]
  pub fn get_component(&self, instance: &str) -> Option<ComponentDefinition> {
    self.instances.get(instance).cloned()
  }

  /// Get a reference to the [ComponentDefinition] map.
  #[must_use]
  pub fn instances(&self) -> &HashMap<String, ComponentDefinition> {
    &self.instances
  }
}

impl TryFrom<&crate::v0::SchematicManifest> for SchematicDefinition {
  type Error = Error;

  fn try_from(manifest: &crate::v0::SchematicManifest) -> Result<Self> {
    let instances: Result<HashMap<String, ComponentDefinition>> = manifest
      .instances
      .iter()
      .map(|(key, val)| Ok((key.clone(), val.try_into()?)))
      .collect();
    let connections: Result<Vec<ConnectionDefinition>> =
      manifest.connections.iter().map(|def| def.try_into()).collect();
    Ok(Self {
      name: manifest.name.clone(),
      instances: instances?,
      connections: connections?,
      providers: manifest.providers.clone(),
      constraints: manifest.constraints.clone().into_iter().collect(),
    })
  }
}

impl TryFrom<SchematicManifest<'_>> for SchematicDefinition {
  type Error = Error;

  fn try_from(manifest: SchematicManifest) -> Result<Self> {
    let def = match manifest {
      SchematicManifest::V0(manifest) => manifest.try_into()?,
    };
    Ok(def)
  }
}

#[derive(Debug, Clone, PartialEq)]
/// A definition of a component used to reference a component registered under a provider.
/// Note: [ComponentDefinition] include embed the concept of a namespace so two identical.
/// components registered on different namespaces will not be equal.
pub struct ComponentDefinition {
  /// The component's name.
  pub name: String,
  /// The namespace the component was registered under.
  pub namespace: String,
  /// Data associated with the component instance.
  pub data: Option<Value>,
}

impl ComponentDefinition {
  /// Quick way to create a [ComponentDefinition] from code. Used mostly in testing.
  #[must_use]
  pub fn new(namespace: &str, name: &str, data: Option<Value>) -> Self {
    Self {
      name: name.to_owned(),
      namespace: namespace.to_owned(),
      data,
    }
  }

  /// Returns the fully qualified ID for the component, i.e. namespace::name.
  #[must_use]
  pub fn id(&self) -> String {
    format!("{}::{}", self.namespace, self.name)
  }
}

impl Display for ComponentDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.id())
  }
}

impl TryFrom<crate::v0::ComponentDefinition> for ComponentDefinition {
  type Error = Error;
  fn try_from(def: crate::v0::ComponentDefinition) -> Result<Self> {
    let (ns, name) = parse_id(&def.id)?;
    Ok(ComponentDefinition {
      namespace: ns.to_owned(),
      name: name.to_owned(),
      data: def.data,
    })
  }
}

impl TryFrom<&crate::v0::ComponentDefinition> for ComponentDefinition {
  type Error = Error;
  fn try_from(def: &crate::v0::ComponentDefinition) -> Result<Self> {
    let (ns, name) = parse_id(&def.id)?;
    Ok(ComponentDefinition {
      namespace: ns.to_owned(),
      name: name.to_owned(),
      data: def.data.clone(),
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A [ConnectionDefinition] defines the link between an upstream and downstream port as well as.
/// the default value to use in the case of an exception.
#[must_use]
pub struct ConnectionDefinition {
  /// The upstream [ConnectionTargetDefinition] (port).
  pub from: ConnectionTargetDefinition,
  /// The downstream [ConnectionTargetDefinition] (port).
  pub to: ConnectionTargetDefinition,
  /// The default data to use in the case of an Error, represented as a JSON string.
  pub default: Option<Value>,
}

impl Hash for ConnectionDefinition {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.from.hash(state);
    self.to.hash(state);
  }
}

impl PartialEq for ConnectionDefinition {
  fn eq(&self, other: &Self) -> bool {
    self.from == other.from && self.to == other.to
  }
}

impl Eq for ConnectionDefinition {}

impl ConnectionDefinition {
  /// Constructor for a [ConnectionDefinition]. This is mostly used in tests,.
  /// the most common way to get a [ConnectionDefinition] is by parsing a manifest.
  pub fn new(from: ConnectionTargetDefinition, to: ConnectionTargetDefinition) -> Self {
    Self {
      from,
      to,
      default: None,
    }
  }

  #[must_use]
  /// Returns true if the [ConnectionDefinition] has a default value set.
  pub fn has_default(&self) -> bool {
    self.default.is_some()
  }

  /// Render default JSON template with the passed message.
  pub fn process_default(&self, err: &str) -> Result<Cow<Value>> {
    let json = self.default.as_ref().ok_or_else(|| Error::NoDefault(self.clone()))?;
    process_default(Cow::Borrowed(json), err)
      .map_err(|e| Error::DefaultsError(self.from.clone(), self.to.clone(), e.to_string()))
  }

  /// Generate a [ConnectionDefinition] from short form syntax. See [wasmflow.com](https://wasmflow.com/docs/configuration/short-form-syntax/) for more info.
  pub fn from_v0_str(s: &str) -> Result<Self> {
    let parsed = crate::parse::parse_connection_v0(s)?;
    (&parsed).try_into()
  }
}

/// Configuration specific to a [ConnectionTargetDefinition].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderData {
  inner: Value,
}

impl SenderData {}

impl From<SenderData> for MessageTransport {
  fn from(v: SenderData) -> Self {
    MessageTransport::success(&v.inner)
  }
}

impl FromStr for SenderData {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    let value: Value = serde_json::from_str(s).map_err(|e| Error::InvalidSenderData(e.to_string()))?;
    Ok(SenderData { inner: value })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A [ConnectionTargetDefinition] is a wrapper around an [Option<PortReference>].
#[must_use]
pub struct ConnectionTargetDefinition {
  target: PortReference,
  data: Option<SenderData>,
}

impl Hash for ConnectionTargetDefinition {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.target.hash(state);
  }
}

impl PartialEq for ConnectionTargetDefinition {
  fn eq(&self, other: &Self) -> bool {
    self.target == other.target
  }
}

impl Eq for ConnectionTargetDefinition {}

impl ConnectionTargetDefinition {
  /// Constructor for a [PortReference]. Used mostly in test code.
  pub fn new<T: AsRef<str>, U: AsRef<str>>(instance: T, port: U) -> Self {
    Self {
      target: PortReference {
        instance: instance.as_ref().to_owned(),
        port: port.as_ref().to_owned(),
      },
      data: None,
    }
  }

  /// Create a [ConnectionTargetDefinition] that points nowhere.
  pub fn sender(config: Option<SenderData>) -> Self {
    Self {
      target: PortReference {
        instance: crate::parse::SENDER_ID.to_owned(),
        port: crate::parse::SENDER_PORT.to_owned(),
      },
      data: config,
    }
  }

  /// Getter for the target's [SenderData].
  #[must_use]
  pub fn get_data(&self) -> Option<&SenderData> {
    self.data.as_ref()
  }

  /// Create a [ConnectionTargetDefinition] with the target set to the passed port.
  pub fn from_port(port: PortReference) -> Self {
    Self {
      target: port,
      data: None,
    }
  }

  #[must_use]
  /// Convenience method to test if the target's instance matches the passed string.
  pub fn matches_instance(&self, instance: &str) -> bool {
    self.target.instance == instance
  }

  #[must_use]
  /// Convenience method to test if the target's port matches the passed string.
  pub fn matches_port(&self, port: &str) -> bool {
    self.target.port == port
  }

  /// Get the target's instance.
  #[must_use]
  pub fn get_instance(&self) -> &str {
    &self.target.instance
  }

  /// Get the target's instance as an owned String.
  #[must_use]
  pub fn get_instance_owned(&self) -> String {
    self.target.instance.clone()
  }

  /// Get the target's port.
  #[must_use]
  pub fn get_port(&self) -> &str {
    &self.target.port
  }

  /// Get the target's port as an owned String if it exists.
  #[must_use]
  pub fn get_port_owned(&self) -> String {
    self.target.port.clone()
  }

  /// Generate a [ConnectionTargetDefinition] from short form syntax. See [wasmflow.com](https://wasmflow.com/docs/configuration/short-form-syntax/) for more info.
  pub fn from_v0_str(s: &str) -> Result<Self> {
    let parsed = crate::parse::parse_connection_target_v0(s)?;
    parsed.try_into()
  }
}

impl TryFrom<&crate::v0::ConnectionDefinition> for ConnectionDefinition {
  type Error = Error;

  fn try_from(def: &crate::v0::ConnectionDefinition) -> Result<Self> {
    let from: ConnectionTargetDefinition = def.from.clone().try_into()?;
    let to: ConnectionTargetDefinition = def.to.clone().try_into()?;
    let default = match &def.default {
      Some(json_str) => {
        Some(parse_default(json_str).map_err(|e| Error::DefaultsError(from.clone(), to.clone(), e.to_string()))?)
      }
      None => None,
    };
    Ok(ConnectionDefinition { from, to, default })
  }
}

impl Display for ConnectionDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} => {}", self.from, self.to)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]

/// A [PortReference] is the link to a port for a specific reference of a component.
pub struct PortReference {
  /// A schematic-wide unique reference that maps to a [ComponentDefinition].
  pub instance: String,
  /// A port on the referenced [ComponentDefinition].
  pub port: String,
}

impl PortReference {
  /// Constructor for a [PortReference]. Used mostly in test code.
  pub fn new<T: AsRef<str>, U: AsRef<str>>(instance: T, port: U) -> Self {
    Self {
      instance: instance.as_ref().to_owned(),
      port: port.as_ref().to_owned(),
    }
  }
}

impl Default for PortReference {
  fn default() -> Self {
    Self {
      instance: "<None>".to_owned(),
      port: "<None>".to_owned(),
    }
  }
}

impl<T, U> From<(T, U)> for PortReference
where
  T: AsRef<str>,
  U: AsRef<str>,
{
  fn from((instance, port): (T, U)) -> Self {
    PortReference {
      instance: instance.as_ref().to_owned(),
      port: port.as_ref().to_owned(),
    }
  }
}

impl Display for ConnectionTargetDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}[{}]", self.target.instance, self.target.port)
  }
}

impl Display for PortReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}[{}]", self.instance, self.port)
  }
}

impl From<&crate::v0::ConnectionTargetDefinition> for PortReference {
  fn from(def: &crate::v0::ConnectionTargetDefinition) -> Self {
    PortReference {
      instance: def.instance.clone(),
      port: def.port.clone(),
    }
  }
}

impl TryFrom<crate::v0::ConnectionTargetDefinition> for ConnectionTargetDefinition {
  type Error = Error;

  fn try_from(def: crate::v0::ConnectionTargetDefinition) -> Result<Self> {
    let data = def.data.map(|json| SenderData { inner: json });
    Ok(ConnectionTargetDefinition {
      target: PortReference {
        instance: def.instance,
        port: def.port,
      },
      data,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test_logger::test]
  fn test_parse_id() -> Result<()> {
    let id = "namespace::component_name";
    let (ns, name) = parse_id(id)?;
    assert_eq!(ns, "namespace");
    assert_eq!(name, "component_name");
    let id = "namespace::subns::component_name";
    let (ns, name) = parse_id(id)?;
    assert_eq!(ns, "namespace");
    assert_eq!(name, "subns::component_name");
    Ok(())
  }
}
