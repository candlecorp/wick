use std::error::Error;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::maps::{FieldMap, OperationMap, TypeMap};
/// The signature of a Wick component, including its input and output types.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct OperationSignature {
  /// The index of the action.
  #[serde(default)]
  pub index: u32,
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

impl OperationSignature {
  /// Create a new [ComponentSignature] with the passed name.
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      ..Default::default()
    }
  }

  /// Add an input port.
  pub fn add_input(mut self, name: impl AsRef<str>, input_type: TypeSignature) -> Self {
    self.inputs.insert(name, input_type);
    self
  }

  /// Add an input port.
  pub fn add_output(mut self, name: impl AsRef<str>, input_type: TypeSignature) -> Self {
    self.outputs.insert(name, input_type);
    self
  }
}

#[derive(Debug, Clone, Copy, PartialEq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[must_use]
#[repr(u32)]
/// The umbrella version of the collection.
pub enum CollectionVersion {
  /// Version 0 Wick collections.
  V0 = 0,
}

impl Default for CollectionVersion {
  fn default() -> Self {
    Self::V0
  }
}

impl From<CollectionVersion> for u32 {
  fn from(v: CollectionVersion) -> Self {
    match v {
      CollectionVersion::V0 => 0,
    }
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Copy)]
#[must_use]
/// The Wick features this collection supports.
pub struct CollectionFeatures {
  /// Whether or not this collection's components accept streaming input or produce streaming output.
  pub streaming: bool,
  /// Whether or not this collection has a persistent state or context.
  pub stateful: bool,
  /// The version of this component.
  pub version: CollectionVersion,
}

impl CollectionFeatures {
  /// Quickly create a v0 feature set.
  pub fn v0(stateful: bool, streaming: bool) -> Self {
    Self {
      streaming,
      stateful,
      version: CollectionVersion::V0,
    }
  }
}

/// Signature for Collections.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct CollectionSignature {
  /// Name of the collection.
  pub name: Option<String>,
  /// Component implementation version.
  pub features: CollectionFeatures,
  /// Schema format version.
  pub format: u32,
  /// Version of the schema.
  pub version: String,
  /// A map of type signatures referenced elsewhere.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub wellknown: Vec<WellKnownSchema>,
  /// A map of type signatures referenced elsewhere.
  #[serde(default, skip_serializing_if = "TypeMap::is_empty")]
  // #[serde(skip)]
  pub types: TypeMap,
  /// A list of [ComponentSignature]s in this collection.
  pub operations: OperationMap,
  /// The component's configuration for this implementation.
  #[serde(default, skip_serializing_if = "TypeMap::is_empty")]
  pub config: TypeMap,
}

impl CollectionSignature {
  /// Create a new [CollectionSignature] with the passed name.
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: Some(name.as_ref().to_owned()),
      ..Default::default()
    }
  }

  #[must_use]
  /// Get the [CollectionSignature] for the requested component.
  pub fn get_component(&self, field: &str) -> Option<&OperationSignature> {
    self.operations.get(field)
  }

  /// Add a [ComponentSignature] to the collection.
  pub fn add_component(mut self, signature: OperationSignature) -> Self {
    self.operations.insert(signature.name.clone(), signature);
    self
  }

  /// Set the version of the [CollectionSignature].
  pub fn version(mut self, version: impl AsRef<str>) -> Self {
    self.version = version.as_ref().to_owned();
    self
  }

  /// Set the format of the [CollectionSignature].
  pub fn format(mut self, format: u32) -> Self {
    self.format = format;
    self
  }

  /// Set the features of the [CollectionSignature].
  pub fn features(mut self, features: CollectionFeatures) -> Self {
    self.features = features;
    self
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
  pub schema: CollectionSignature,
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
  /// A collection.
  Collection(CollectionSignature),
}

impl HostedType {
  /// Get the name of the [HostedType] regardless of kind.
  #[must_use]
  pub fn get_name(&self) -> &Option<String> {
    match self {
      HostedType::Collection(s) => &s.name,
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
  /// A custom type name.
  Custom(String),
  /// An internal type.
  Internal(InternalType),
  /// A reference to another type.
  Ref {
    #[serde(rename = "ref")]
    /// The reference string
    reference: String,
  },
  /// A stream type
  Stream {
    /// The inner type
    item: Box<TypeSignature>,
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
  /// A type representing a link to another collection.
  Link {
    /// The schemas that must be provided with the linked collection.
    #[serde(default)]
    schemas: Vec<String>,
  },
  /// A JSON-like key/value map.
  Struct,
  /// An inline, anonymous struct interface.
  AnonymousStruct(
    /// A map of field names to their types.
    FieldMap,
  ),
}
#[derive(Debug)]
/// Error returned when attempting to convert an invalid source into a Wick type.
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

/// Internal types for use within the Wick runtime
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
