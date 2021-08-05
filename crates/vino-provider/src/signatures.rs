use std::fmt::Display;

use serde::{
  Deserialize,
  Serialize,
};

/// The signature of a Vino component, including its input and output types.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ComponentSignature {
  /// The name of the component.
  pub name: String,
  /// A list of input signatures.
  pub inputs: Vec<PortSignature>,
  /// A list of output signatures.
  pub outputs: Vec<PortSignature>,
}

/// The signature of an individual port.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PortSignature {
  /// Name of the port.
  pub name: String,

  /// The data type of the port.
  // TODO: Need to turn this into a more complex representation of port types
  pub type_string: String,
}

impl PortSignature {
  /// Constructor.
  #[must_use]
  pub fn new(name: String, type_string: String) -> Self {
    Self { name, type_string }
  }
}

impl From<(&str, &str)> for PortSignature {
  fn from(tup: (&str, &str)) -> Self {
    let (name, type_string) = tup;
    Self {
      name: name.to_owned(),
      type_string: type_string.to_owned(),
    }
  }
}

impl Display for PortSignature {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("{}: {}", self.name, self.type_string))
  }
}

/// Signature for Providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProviderSignature {
  /// Name of the provider.
  pub name: String,
  /// A list of [ComponentSignature]s the provider hosts.
  pub components: Vec<ComponentSignature>,
}

/// Signature for schematics, their ports, and their providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SchematicSignature {
  /// Name of the schematic.
  pub name: String,
  /// A list of input ports.
  pub inputs: Vec<PortSignature>,
  /// A list of output ports.
  pub outputs: Vec<PortSignature>,
  /// A list of providers running on the schematic.
  pub providers: Vec<ProviderSignature>,
}

/// An enum representing the types of components that can be hosted.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum HostedType {
  /// A hosted component.
  Component(ComponentSignature),
  /// A hosted schematic.
  Schematic(SchematicSignature),
}

impl HostedType {
  /// Get the name of the [HostedType] regardless of kind.
  #[must_use]
  pub fn get_name(&self) -> &str {
    match self {
      HostedType::Component(c) => &c.name,
      HostedType::Schematic(s) => &s.name,
    }
  }
}
