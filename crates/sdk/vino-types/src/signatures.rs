use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::iter::FromIterator;
use std::str::FromStr;

use serde::{
  Deserialize,
  Serialize,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
#[must_use]
/// A HashMap of type names to their signature.
pub struct TypeMap(HashMap<String, TypeSignature>);

impl TypeMap {
  /// Constructor for [TypeMap]
  pub fn new() -> Self {
    Self(HashMap::new())
  }
}

impl MapWrapper<TypeSignature> for TypeMap {
  fn get_inner_owned(self) -> HashMap<String, TypeSignature> {
    self.0
  }

  fn get_inner(&self) -> &HashMap<String, TypeSignature> {
    &self.0
  }

  fn get_inner_mut(&mut self) -> &mut HashMap<String, TypeSignature> {
    &mut self.0
  }

  fn new() -> Self {
    Self(HashMap::new())
  }
}

impl From<HashMap<String, TypeSignature>> for TypeMap {
  fn from(map: HashMap<String, TypeSignature>) -> Self {
    Self(map)
  }
}

impl TryFrom<Vec<(&str, &str)>> for TypeMap {
  type Error = ParseError;

  fn try_from(list: Vec<(&str, &str)>) -> Result<Self, ParseError> {
    let mut map = TypeMap::new();
    for (k, v) in list {
      map.insert(k.to_owned(), TypeSignature::from_str(v)?);
    }
    Ok(map)
  }
}

impl FromIterator<(String, TypeSignature)> for TypeMap {
  fn from_iter<T: IntoIterator<Item = (String, TypeSignature)>>(iter: T) -> Self {
    let mut map: HashMap<String, TypeSignature> = HashMap::new();
    for (k, v) in iter {
      map.insert(k, v);
    }
    Self(map)
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
#[must_use]
/// A HashMap of struct names to their signature.
pub struct StructMap(pub HashMap<String, StructSignature>);

impl From<HashMap<String, StructSignature>> for StructMap {
  fn from(map: HashMap<String, StructSignature>) -> Self {
    Self(map)
  }
}

impl StructMap {
  /// Constructor for [StructMap]
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  #[doc(hidden)]
  pub fn todo() -> Self {
    let mut map = HashMap::new();
    map.insert(
      "TODO".to_owned(),
      StructSignature {
        name: "todo".to_owned(),
        fields: HashMap::new().into(),
      },
    );
    Self(map)
  }
}

impl MapWrapper<StructSignature> for StructMap {
  fn get_inner_owned(self) -> HashMap<String, StructSignature> {
    self.0
  }

  fn get_inner(&self) -> &HashMap<String, StructSignature> {
    &self.0
  }

  fn get_inner_mut(&mut self) -> &mut HashMap<String, StructSignature> {
    &mut self.0
  }

  fn new() -> Self {
    Self(HashMap::new())
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
#[must_use]
/// A HashMap of provider names to their signature.
pub struct ProviderMap(pub HashMap<String, ProviderSignature>);

impl From<HashMap<String, ProviderSignature>> for ProviderMap {
  fn from(map: HashMap<String, ProviderSignature>) -> Self {
    Self(map)
  }
}

impl MapWrapper<ProviderSignature> for ProviderMap {
  fn get_inner_owned(self) -> HashMap<String, ProviderSignature> {
    self.0
  }

  fn get_inner(&self) -> &HashMap<String, ProviderSignature> {
    &self.0
  }

  fn get_inner_mut(&mut self) -> &mut HashMap<String, ProviderSignature> {
    &mut self.0
  }

  fn new() -> Self {
    Self(HashMap::new())
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
#[must_use]
/// A HashMap of component names to their signature.
pub struct ComponentMap(pub HashMap<String, ComponentSignature>);

impl MapWrapper<ComponentSignature> for ComponentMap {
  fn get_inner_owned(self) -> HashMap<String, ComponentSignature> {
    self.0
  }

  fn get_inner(&self) -> &HashMap<String, ComponentSignature> {
    &self.0
  }

  fn get_inner_mut(&mut self) -> &mut HashMap<String, ComponentSignature> {
    &mut self.0
  }

  fn new() -> Self {
    Self(HashMap::new())
  }
}

impl From<HashMap<String, ComponentSignature>> for ComponentMap {
  fn from(map: HashMap<String, ComponentSignature>) -> Self {
    Self(map)
  }
}

/// Utility functions for HashMap wrappers.
pub trait MapWrapper<T>
where
  Self: Sized,
{
  /// Constructor for the map.
  fn new() -> Self;
  /// Get the inner HashMap.
  fn get_inner_owned(self) -> HashMap<String, T>;

  /// Get a reference to the inner HashMap.
  fn get_inner(&self) -> &HashMap<String, T>;

  /// Get a mutable reference to the inner HashMap.
  fn get_inner_mut(&mut self) -> &mut HashMap<String, T>;

  #[must_use]
  /// Return a list of names in the inner HashMap.
  fn names(&self) -> Vec<String> {
    self.get_inner().keys().cloned().collect()
  }

  #[must_use]
  /// Return true if the inner HashMap is empty.
  fn is_empty(&self) -> bool {
    self.get_inner().is_empty()
  }

  /// Return the inner HashMap.
  #[must_use]
  fn into_inner(self) -> HashMap<String, T> {
    self.get_inner_owned()
  }

  /// Return a reference to the inner HashMap.
  #[must_use]
  fn inner(&self) -> &HashMap<String, T> {
    &self.get_inner()
  }

  #[must_use]
  /// Get the value for the requested field.
  fn get<K: AsRef<str>>(&self, field: K) -> Option<&T> {
    self.get_inner().get(field.as_ref())
  }

  /// Insert a [T] into the inner map.
  fn insert<K: AsRef<str>>(&mut self, field: K, value: T) {
    self
      .get_inner_mut()
      .insert(field.as_ref().to_owned(), value);
  }
}

/// The signature of a Vino component, including its input and output types.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct ComponentSignature {
  /// The name of the component.
  pub name: String,
  /// The component's inputs.
  pub inputs: TypeMap,
  /// The component's outputs.
  pub outputs: TypeMap,
}

/// Signature for Providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct ProviderSignature {
  /// Name of the provider.
  pub name: String,
  /// A map of type signatures referenced elsewhere.
  pub types: StructMap,
  /// A list of [ComponentSignature]s the provider hosts.
  pub components: ComponentMap,
}

impl ProviderSignature {
  #[must_use]
  /// Get the [ProviderSignature] for the requested field.
  pub fn get_component<T: AsRef<str>>(&self, field: T) -> Option<&ComponentSignature> {
    self.components.get(field.as_ref())
  }
}

/// Signatures of struct-like type definitions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct StructSignature {
  /// The name of the struct.
  pub name: String,
  /// The fields in this struct.
  pub fields: TypeMap,
}

/// An enum representing the types of components that can be hosted.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[must_use]
pub enum HostedType {
  /// A provider.
  Provider(ProviderSignature),
}

impl HostedType {
  /// Get the name of the [HostedType] regardless of kind.
  #[must_use]
  pub fn get_name(&self) -> &str {
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
  /// Raw value to be processed later.
  Raw,
  /// Any valid value.
  Value,
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
  /// A type representing a ProviderLink
  Link {
    /// The provider ID
    provider: Option<String>,
  },
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
      "raw" => Self::Raw,
      "value" => Self::Value,
      "string" => Self::String,
      "datetime" => Self::Datetime,
      _ => return Err(ParseError(s.to_owned())),
    };
    Ok(t)
  }
}
