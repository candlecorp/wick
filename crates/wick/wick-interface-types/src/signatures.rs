use std::error::Error;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Field {
  pub name: String,
  #[serde(rename = "type")]
  #[cfg_attr(feature = "yaml", serde(with = "serde_yaml::with::singleton_map"))]
  pub ty: TypeSignature,
}

impl Field {
  pub fn new(name: impl AsRef<str>, ty: TypeSignature) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      ty,
    }
  }
}

impl std::fmt::Display for Field {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.name);
    f.write_str(": ")?;
    self.ty.fmt(f)
  }
}

/// The signature of a Wick component, including its input and output types.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct OperationSignature {
  /// The name of the component.
  #[serde(default)]
  pub name: String,
  /// The component's inputs.
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub inputs: Vec<Field>,
  /// The component's outputs.
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub outputs: Vec<Field>,
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
  pub fn add_input(mut self, name: impl AsRef<str>, ty: TypeSignature) -> Self {
    self.inputs.push(Field::new(name, ty));
    self
  }

  /// Add an input port.
  pub fn add_output(mut self, name: impl AsRef<str>, ty: TypeSignature) -> Self {
    self.outputs.push(Field::new(name, ty));
    self
  }
}

#[derive(Debug, Clone, Copy, PartialEq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[must_use]
#[repr(u32)]
/// The umbrella version of the collection.
pub enum ComponentVersion {
  /// Version 0 Wick collections.
  V0 = 0,
}

impl Default for ComponentVersion {
  fn default() -> Self {
    Self::V0
  }
}

impl From<ComponentVersion> for u32 {
  fn from(v: ComponentVersion) -> Self {
    match v {
      ComponentVersion::V0 => 0,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
/// The Wick features this collection supports.
pub struct ComponentMetadata {
  /// Version of the component.
  pub version: Option<String>,
}

impl ComponentMetadata {
  /// Quickly create a v0 feature set.
  pub fn v0() -> Self {
    Self { version: None }
  }
}

impl Default for ComponentMetadata {
  fn default() -> Self {
    Self::v0()
  }
}

/// Signature for Collections.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct ComponentSignature {
  /// Name of the collection.
  pub name: Option<String>,

  /// The format of the component signature.
  pub format: ComponentVersion,

  /// Component implementation version.
  #[serde(default)]
  pub metadata: ComponentMetadata,

  /// A map of type signatures referenced elsewhere.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub wellknown: Vec<WellKnownSchema>,

  /// A map of type signatures referenced elsewhere.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub types: Vec<TypeDefinition>,

  /// A list of [OperationSignature]s in this component.
  pub operations: Vec<OperationSignature>,
  /// The component's configuration for this implementation.

  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub config: Vec<TypeDefinition>,
}

impl ComponentSignature {
  /// Create a new [ComponentSignature] with the passed name.
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: Some(name.as_ref().to_owned()),
      ..Default::default()
    }
  }

  #[must_use]
  /// Get the [ComponentSignature] for the requested component.
  pub fn get_component(&self, field: &str) -> Option<&OperationSignature> {
    self.operations.iter().find(|op| op.name == field)
  }

  /// Add a [ComponentSignature] to the collection.
  pub fn add_component(mut self, signature: OperationSignature) -> Self {
    self.operations.push(signature);
    self
  }

  /// Set the version of the [ComponentSignature].
  pub fn version(mut self, version: impl AsRef<str>) -> Self {
    self.metadata.version = Some(version.as_ref().to_owned());
    self
  }

  /// Set the format of the [ComponentSignature].
  pub fn format(mut self, format: ComponentVersion) -> Self {
    self.format = format;
    self
  }

  /// Set the features of the [ComponentSignature].
  pub fn metadata(mut self, features: ComponentMetadata) -> Self {
    self.metadata = features;
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
  pub schema: ComponentSignature,
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
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub fields: Vec<Field>,
}

impl StructSignature {
  /// Constructor for [StructSignature]
  pub fn new<T: AsRef<str>>(name: T, fields: Vec<Field>) -> Self {
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
  Collection(ComponentSignature),
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
    #[serde(rename = "type")]
    #[cfg_attr(feature = "yaml", serde(with = "serde_yaml::with::singleton_map"))]
    ty: Box<TypeSignature>,
  },
  /// A list type
  List {
    /// The type of the list's elements
    #[serde(rename = "type")]
    #[cfg_attr(feature = "yaml", serde(with = "serde_yaml::with::singleton_map"))]
    ty: Box<TypeSignature>,
  },
  /// A type representing an optional value.
  Optional {
    /// The actual type that is optional.
    #[serde(rename = "type")]
    #[cfg_attr(feature = "yaml", serde(with = "serde_yaml::with::singleton_map"))]
    ty: Box<TypeSignature>,
  },
  /// A HashMap-like type.
  Map {
    /// The type of the map's keys.
    #[cfg_attr(feature = "yaml", serde(with = "serde_yaml::with::singleton_map"))]
    key: Box<TypeSignature>,
    /// The type of the map's values.
    #[cfg_attr(feature = "yaml", serde(with = "serde_yaml::with::singleton_map"))]
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
    /// A list of fields in the struct.
    Vec<Field>,
  ),
}

impl std::fmt::Display for TypeSignature {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TypeSignature::I8 => f.write_str("i8"),
      TypeSignature::I16 => f.write_str("i16"),
      TypeSignature::I32 => f.write_str("i32"),
      TypeSignature::I64 => f.write_str("i64"),
      TypeSignature::U8 => f.write_str("u8"),
      TypeSignature::U16 => f.write_str("u16"),
      TypeSignature::U32 => f.write_str("u32"),
      TypeSignature::U64 => f.write_str("u64"),
      TypeSignature::F32 => f.write_str("f32"),
      TypeSignature::F64 => f.write_str("f64"),
      TypeSignature::Bool => f.write_str("bool"),
      TypeSignature::String => f.write_str("string"),
      TypeSignature::Datetime => f.write_str("datetime"),
      TypeSignature::Bytes => f.write_str("bytes"),
      TypeSignature::Value => f.write_str("value"),
      TypeSignature::Custom(_) => todo!(),
      TypeSignature::Internal(_) => todo!(),
      TypeSignature::Ref { reference } => todo!(),
      TypeSignature::Stream { ty } => todo!(),
      TypeSignature::List { ty } => todo!(),
      TypeSignature::Optional { ty } => todo!(),
      TypeSignature::Map { key, value } => todo!(),
      TypeSignature::Link { schemas } => todo!(),
      TypeSignature::Struct => todo!(),
      TypeSignature::AnonymousStruct(_) => todo!(),
    }
  }
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
  OperationInput,
}

impl FromStr for InternalType {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let t = match s {
      "component_input" => Self::OperationInput,
      _ => return Err(ParseError(s.to_owned())),
    };
    Ok(t)
  }
}
