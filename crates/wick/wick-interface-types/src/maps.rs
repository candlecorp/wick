use std::collections::HashMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::signatures::{ComponentSignature, OperationSignature, ParseError, TypeSignature};
use crate::TypeDefinition;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
#[must_use]
/// A HashMap from collection names to their signatures.
pub struct CollectionMap(pub HashMap<String, ComponentSignature>);

impl From<HashMap<String, ComponentSignature>> for CollectionMap {
  fn from(map: HashMap<String, ComponentSignature>) -> Self {
    Self(map)
  }
}

impl CollectionMap {
  kv_impl! {ComponentSignature, pub}
}
