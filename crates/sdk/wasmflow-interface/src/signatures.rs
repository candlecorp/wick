use std::error::Error;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::maps::{ComponentMap, FieldMap, TypeMap};
/// The signature of a Wasmflow component, including its input and output types.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct ComponentSignature {
  /// The name of the component.
  #[serde(default)]
  pub name: String,
  /// The component's inputs.
  #[serde(default)]
  pub inputs: FieldMap,
  /// The component's outputs.
  #[serde(default)]
  pub outputs: FieldMap,
}

impl ComponentSignature {
  /// Create a new [ComponentSignature] with the passed name.
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      ..Default::default()
    }
  }
}

/// Signature for Providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct ProviderSignature {
  /// Name of the provider.
  pub name: Option<String>,
  /// Schema format version.
  pub format: u32,
  /// Version of the schema.
  pub version: String,
  /// A map of type signatures referenced elsewhere.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub wellknown: Vec<WellKnownSchema>,
  /// A map of type signatures referenced elsewhere.
  #[serde(default, skip_serializing_if = "TypeMap::is_empty")]
  pub types: TypeMap,
  /// A list of [ComponentSignature]s the provider hosts.
  pub components: ComponentMap,
  /// The component's configuration for this implementation.
  #[serde(default, skip_serializing_if = "TypeMap::is_empty")]
  pub config: TypeMap,
}

impl ProviderSignature {
  /// Create a new [ProviderSignature] with the passed name.
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: Some(name.as_ref().to_owned()),
      ..Default::default()
    }
  }

  #[must_use]
  /// Get the [ProviderSignature] for the requested field.
  pub fn get_component<T: AsRef<str>>(&self, field: T) -> Option<&ComponentSignature> {
    self.components.get(field.as_ref())
  }
}

/// An entry from a well-known schema
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WellKnownSchema {
  /// The capability the schema provides.
  pub capabilities: Vec<String>,
  /// The location where you can find and validate the schema.
  pub url: String,
  /// The schema itself.
  pub schema: ProviderSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
/// A valid type definition.
#[serde(tag = "type")]
pub enum TypeDefinition {
  /// A struct definition.
  #[serde(rename = "struct")]
  Struct(StructSignature),
  /// An enum definition.
  #[serde(rename = "enum")]
  Enum(EnumSignature),
}

/// Signatures of enum type definitions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct EnumSignature {
  /// The name of the enum.
  pub name: String,
  /// The variants in the enum.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub values: Vec<EnumVariant>,
}

impl EnumSignature {
  /// Constructor for [EnumSignature]
  pub fn new<T: AsRef<str>>(name: T, values: Vec<EnumVariant>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      values,
    }
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
/// An enum variant definition
pub struct EnumVariant {
  /// The name of the variant.
  pub name: String,
  /// The index of the variant.
  pub index: u32,
}

impl EnumVariant {
  /// Constructor for [EnumVariant]
  pub fn new<T: AsRef<str>>(name: T, index: u32) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      index,
    }
  }
}

/// Signatures of struct-like type definitions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct StructSignature {
  /// The name of the struct.
  pub name: String,
  /// The fields in this struct.
  pub fields: FieldMap,
}

impl StructSignature {
  /// Constructor for [StructSignature]
  pub fn new<T: AsRef<str>>(name: T, fields: FieldMap) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      fields,
    }
  }
}

/// An enum representing the types of components that can be hosted.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[must_use]
pub enum HostedType {
  /// A provider.
  Provider(ProviderSignature),
}

impl HostedType {
  /// Get the name of the [HostedType] regardless of kind.
  #[must_use]
  pub fn get_name(&self) -> &Option<String> {
    match self {
      HostedType::Provider(s) => &s.name,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
#[must_use]
/// Enum of valid types.
pub enum TypeSignature {
  /// I8 type.
  I8,
  /// I16 type.
  I16,
  /// I32 type.
  I32,
  /// I64 type.
  I64,
  /// u8 type.
  U8,
  /// u16 type.
  U16,
  /// u32 type.
  U32,
  /// u64 type.
  U64,
  /// f32 type.
  F32,
  /// f64 type.
  F64,
  /// Boolean type.
  Bool,
  /// String type.
  String,
  /// Date type.
  Datetime,
  /// Raw bytes.
  Bytes,
  /// Any valid value.
  Value,
  /// An internal type.
  Internal(InternalType),
  /// A reference to another type.
  Ref {
    #[serde(rename = "ref")]
    /// The reference string
    reference: String,
  },
  /// A list type
  List {
    /// The type of the list's elements
    element: Box<TypeSignature>,
  },
  /// A type representing an optional value.
  Optional {
    /// The actual type that is optional.
    option: Box<TypeSignature>,
  },
  /// A HashMap-like type.
  Map {
    /// The type of the map's keys.
    key: Box<TypeSignature>,
    /// The type of the map's values.
    value: Box<TypeSignature>,
  },
  /// A type representing a ProviderLink.
  Link {
    /// The schemas that must be provided with the linked provider.
    #[serde(default)]
    schemas: Vec<String>,
  },
  /// A JSON-like key/value map.
  Struct,
}
#[derive(Debug)]
/// Error returned when attempting to convert an invalid string into a [TypeSignature].
pub struct ParseError(String);
impl Error for ParseError {}
impl std::fmt::Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Could not parse {} into a TypeSignature.", self.0)
  }
}

impl FromStr for TypeSignature {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let t = match s {
      "i8" => Self::I8,
      "i16" => Self::I16,
      "i32" => Self::I32,
      "i64" => Self::I64,
      "u8" => Self::U8,
      "u16" => Self::U16,
      "u32" => Self::U32,
      "u64" => Self::U64,
      "f32" => Self::F32,
      "f64" => Self::F64,
      "bool" => Self::Bool,
      "bytes" => Self::Bytes,
      "value" => Self::Value,
      "string" => Self::String,
      "datetime" => Self::Datetime,
      _ => return Err(ParseError(s.to_owned())),
    };
    Ok(t)
  }
}

/// Internal types for use within the Wasmflow runtime
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
#[serde(tag = "id")]
pub enum InternalType {
  /// Represents a complete set of component inputs
  #[serde(rename = "__input__")]
  ComponentInput,
}

impl FromStr for InternalType {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let t = match s {
      "component_input" => Self::ComponentInput,
      _ => return Err(ParseError(s.to_owned())),
    };
    Ok(t)
  }
}
