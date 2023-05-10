use std::any::TypeId;
use std::error::Error;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{contents_equal, is_false, TypeDefinition};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Field {
  /// The name of the field.
  pub name: String,

  /// The type of the field.
  #[serde(rename = "type")]
  #[cfg_attr(feature = "parser", serde(deserialize_with = "crate::signatures::type_signature"))]
  #[cfg_attr(
    feature = "yaml",
    serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
  )]
  pub ty: TypeSignature,

  /// Whether the field is required.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub default: Option<serde_json::Value>,

  /// Whether the field is required.
  #[serde(default, skip_serializing_if = "is_false")]
  pub required: bool,

  /// The description of the field.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

impl Field {
  pub fn new(name: impl AsRef<str>, ty: TypeSignature) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      description: None,
      default: None,
      required: true,
      ty,
    }
  }

  /// Get the name of the field
  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Get the type of the field
  pub fn ty(&self) -> &TypeSignature {
    &self.ty
  }

  /// Get the description of the field
  #[must_use]
  pub fn description(&self) -> Option<&str> {
    self.description.as_deref()
  }

  /// Get the default value of the field
  #[must_use]
  pub fn default(&self) -> Option<&serde_json::Value> {
    self.default.as_ref()
  }

  /// Get whether the field is required
  #[must_use]
  pub fn required(&self) -> bool {
    self.required
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq)]
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

impl PartialEq for OperationSignature {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
      && contents_equal(&self.inputs, &other.inputs)
      && contents_equal(&self.outputs, &other.outputs)
  }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[must_use]
#[repr(u32)]
/// The umbrella version of the component.
pub enum ComponentVersion {
  /// Version 0 Wick components.
  V0 = 0,
  /// Version 1 Wick components.
  V1 = 1,
}

impl Default for ComponentVersion {
  fn default() -> Self {
    Self::V1
  }
}

impl From<ComponentVersion> for u32 {
  fn from(v: ComponentVersion) -> Self {
    match v {
      ComponentVersion::V0 => 0,
      ComponentVersion::V1 => 1,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[must_use]
/// The Wick features this collection supports.
pub struct ComponentMetadata {
  /// Version of the component.
  #[serde(skip_serializing_if = "Option::is_none  ")]
  pub version: Option<String>,
}

impl ComponentMetadata {
  pub fn new(version: impl AsRef<str>) -> Self {
    Self {
      version: Some(version.as_ref().to_owned()),
    }
  }
}

/// Signature for Collections.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq)]
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
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub operations: Vec<OperationSignature>,
  /// The component's configuration for this implementation.

  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub config: Vec<TypeDefinition>,
}

impl PartialEq for ComponentSignature {
  fn eq(&self, other: &Self) -> bool {
    let types_equal = contents_equal(&self.types, &other.types);
    let operations_equal = contents_equal(&self.operations, &other.operations);
    let config_equal = contents_equal(&self.config, &other.config);
    let wellknown_equal = contents_equal(&self.wellknown, &other.wellknown);

    self.format == other.format
      && self.metadata == other.metadata
      && types_equal
      && operations_equal
      && config_equal
      && wellknown_equal
  }
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
  /// Get the [OperationSignature] for the requested component.
  pub fn get_operation(&self, field: &str) -> Option<&OperationSignature> {
    self.operations.iter().find(|op| op.name == field)
  }

  /// Add a [OperationSignature] to the collection.
  pub fn add_operation(mut self, signature: OperationSignature) -> Self {
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WellKnownSchema {
  /// The capability the schema provides.
  pub capabilities: Vec<String>,
  /// The location where you can find and validate the schema.
  pub url: String,
  /// The schema itself.
  pub schema: ComponentSignature,
}

/// An enum representing the types of components that can be hosted.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[must_use]
pub enum HostedType {
  /// A collection.
  Component(ComponentSignature),
}

impl HostedType {
  /// Get the name of the [HostedType] regardless of kind.
  #[must_use]
  pub fn get_name(&self) -> &Option<String> {
    match self {
      HostedType::Component(s) => &s.name,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
  /// A custom type name.
  Custom(String),
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
    #[cfg_attr(
      feature = "parser",
      serde(deserialize_with = "crate::signatures::box_type_signature")
    )]
    #[cfg_attr(
      feature = "yaml",
      serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
    )]
    ty: Box<TypeSignature>,
  },
  /// A list type
  List {
    /// The type of the list's elements
    #[serde(rename = "type")]
    #[cfg_attr(
      feature = "parser",
      serde(deserialize_with = "crate::signatures::box_type_signature")
    )]
    #[cfg_attr(
      feature = "yaml",
      serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
    )]
    ty: Box<TypeSignature>,
  },
  /// A type representing an optional value.
  Optional {
    /// The actual type that is optional.
    #[serde(rename = "type")]
    #[cfg_attr(
      feature = "parser",
      serde(deserialize_with = "crate::signatures::box_type_signature")
    )]
    #[cfg_attr(
      feature = "yaml",
      serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
    )]
    ty: Box<TypeSignature>,
  },
  /// A HashMap-like type.
  Map {
    /// The type of the map's keys.
    #[cfg_attr(
      feature = "parser",
      serde(deserialize_with = "crate::signatures::box_type_signature")
    )]
    #[cfg_attr(
      feature = "yaml",
      serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
    )]
    key: Box<TypeSignature>,
    /// The type of the map's values.
    #[cfg_attr(
      feature = "parser",
      serde(deserialize_with = "crate::signatures::box_type_signature")
    )]
    #[cfg_attr(
      feature = "yaml",
      serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
    )]
    value: Box<TypeSignature>,
  },
  /// A type representing a link to another collection.
  Link {
    /// The schemas that must be provided with the linked collection.
    #[serde(default)]
    schemas: Vec<String>,
  },
  /// A JSON-like key/value map.
  Object,
  /// An inline, anonymous struct interface.
  AnonymousStruct(
    /// A list of fields in the struct.
    Vec<Field>,
  ),
}

fn stringify<S>(x: &str, s: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  println!("{:?}", x);
  s.serialize_str(x)
}

impl TypeSignature {
  #[must_use]
  #[cfg(feature = "typeid")]
  pub fn to_type_id(&self) -> TypeId {
    match self {
      TypeSignature::I8 => TypeId::of::<i8>(),
      TypeSignature::I16 => TypeId::of::<i16>(),
      TypeSignature::I32 => TypeId::of::<i32>(),
      TypeSignature::I64 => TypeId::of::<i64>(),
      TypeSignature::U8 => TypeId::of::<u8>(),
      TypeSignature::U16 => TypeId::of::<u16>(),
      TypeSignature::U32 => TypeId::of::<u32>(),
      TypeSignature::U64 => TypeId::of::<u64>(),
      TypeSignature::F32 => TypeId::of::<f32>(),
      TypeSignature::F64 => TypeId::of::<f64>(),
      TypeSignature::Bool => TypeId::of::<bool>(),
      TypeSignature::String => TypeId::of::<String>(),
      TypeSignature::Datetime => TypeId::of::<String>(),
      TypeSignature::Bytes => TypeId::of::<Vec<u8>>(),
      TypeSignature::Custom(_) => TypeId::of::<serde_json::Value>(),
      TypeSignature::Ref { reference } => unimplemented!(),
      TypeSignature::Stream { ty } => unimplemented!(),
      TypeSignature::List { ty } => TypeId::of::<Vec<Box<dyn std::any::Any>>>(),
      TypeSignature::Optional { ty } => TypeId::of::<Option<Box<dyn std::any::Any>>>(),
      TypeSignature::Map { key, value } => {
        TypeId::of::<std::collections::HashMap<Box<dyn std::any::Any>, Box<dyn std::any::Any>>>()
      }
      TypeSignature::Link { schemas } => TypeId::of::<serde_json::Value>(),
      TypeSignature::Object => TypeId::of::<serde_json::Value>(),
      TypeSignature::AnonymousStruct(_) => unimplemented!(),
    }
  }
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
      TypeSignature::Custom(v) => f.write_str(v),
      TypeSignature::Ref { reference } => todo!(),
      TypeSignature::Stream { ty } => todo!(),
      TypeSignature::List { ty } => write!(f, "{}[]", ty),
      TypeSignature::Optional { ty } => write!(f, "{}?", ty),
      TypeSignature::Map { key, value } => write!(f, "{{{}:{}}}", key, value),
      TypeSignature::Link { schemas } => todo!(),
      TypeSignature::Object => f.write_str("object"),
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

#[cfg(feature = "parser")]
impl FromStr for TypeSignature {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    crate::parser::parse(s).map_err(|e| ParseError(s.to_owned()))
  }
}

fn is_valid_typeid(name: &str) -> bool {
  name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(feature = "parser")]
pub(crate) fn type_signature<'de, D>(deserializer: D) -> Result<TypeSignature, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct TypeSignatureVisitor;

  impl<'de> serde::de::Visitor<'de> for TypeSignatureVisitor {
    type Value = TypeSignature;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a TypeSignature definition")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      TypeSignature::from_str(s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      TypeSignature::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
  }

  deserializer.deserialize_any(TypeSignatureVisitor)
}

#[cfg(feature = "parser")]
pub(crate) fn box_type_signature<'de, D>(deserializer: D) -> Result<Box<TypeSignature>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct TypeSignatureVisitor;

  impl<'de> serde::de::Visitor<'de> for TypeSignatureVisitor {
    type Value = Box<TypeSignature>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a TypeSignature definition")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      TypeSignature::from_str(s)
        .map(Box::new)
        .map_err(|e| serde::de::Error::custom(e.to_string()))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      TypeSignature::deserialize(serde::de::value::MapAccessDeserializer::new(map)).map(Box::new)
    }
  }

  deserializer.deserialize_any(TypeSignatureVisitor)
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  #[test]
  fn test_decode() -> Result<()> {
    let ty: TypeSignature = serde_json::from_str(r#""object""#)?;
    assert_eq!(ty, TypeSignature::Object);
    let ty: Field = serde_json::from_str(r#"{"name": "foo", "type": "object"}"#)?;
    assert_eq!(ty.name, "foo");
    assert_eq!(ty.ty, TypeSignature::Object);
    Ok(())
  }
}
