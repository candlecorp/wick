use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::signatures::{
  ComponentSignature, ParseError, ProviderSignature, StructSignature, TypeSignature,
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

  #[must_use]
  /// Returns the number of fields in the map.
  fn len(&self) -> usize {
    self.get_inner().len()
  }
}
