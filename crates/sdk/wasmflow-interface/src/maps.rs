use std::collections::HashMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::signatures::{ComponentSignature, ParseError, ProviderSignature, TypeSignature};
use crate::TypeDefinition;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
#[must_use]
/// A HashMap of type names to their signature.
pub struct FieldMap(HashMap<String, TypeSignature>);

impl FieldMap {
  /// Constructor for [TypeMap]
  pub fn new() -> Self {
    Self(HashMap::new())
  }
}

impl FieldMap {
  wasmflow_macros::kv_impl! {TypeSignature, pub}
}

impl From<HashMap<String, TypeSignature>> for FieldMap {
  fn from(map: HashMap<String, TypeSignature>) -> Self {
    Self(map)
  }
}

impl TryFrom<Vec<(&str, &str)>> for FieldMap {
  type Error = ParseError;

  fn try_from(list: Vec<(&str, &str)>) -> Result<Self, ParseError> {
    let mut map = FieldMap::new();
    for (k, v) in list {
      map.insert(k, TypeSignature::from_str(v)?);
    }
    Ok(map)
  }
}

impl FromIterator<(String, TypeSignature)> for FieldMap {
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
pub struct TypeMap(pub HashMap<String, TypeDefinition>);

impl From<HashMap<String, TypeDefinition>> for TypeMap {
  fn from(map: HashMap<String, TypeDefinition>) -> Self {
    Self(map)
  }
}

impl TypeMap {
  /// Constructor for [TypeMap]
  pub fn new() -> Self {
    Self(HashMap::new())
  }
  wasmflow_macros::kv_impl! {TypeDefinition, pub}
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

impl ProviderMap {
  wasmflow_macros::kv_impl! {ProviderSignature, pub}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
#[must_use]
/// A HashMap of component names to their signature.
pub struct ComponentMap(pub HashMap<String, ComponentSignature>);

impl ComponentMap {
  wasmflow_macros::kv_impl! {ComponentSignature, pub}
}

impl From<HashMap<String, ComponentSignature>> for ComponentMap {
  fn from(map: HashMap<String, ComponentSignature>) -> Self {
    Self(map)
  }
}

impl From<HashMap<&str, ComponentSignature>> for ComponentMap {
  fn from(map: HashMap<&str, ComponentSignature>) -> Self {
    Self(map.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
  }
}
