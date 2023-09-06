use serde::{Deserialize, Serialize};

use crate::{is_false, Type};

/// Signatures of enum type definitions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq)]
#[must_use]
#[non_exhaustive]
pub struct UnionDefinition {
  /// The name of the enum.
  pub name: String,
  /// The variants in the enum.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub types: Vec<Type>,
  /// The optional description of the enum.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Whether this type is imported.
  #[serde(default, skip_serializing_if = "is_false")]
  pub imported: bool,
}

impl UnionDefinition {
  /// Constructor for [UnionDefinition]
  pub fn new<T: Into<String>>(name: T, types: Vec<Type>, description: Option<String>) -> Self {
    Self {
      name: name.into(),
      types,
      imported: false,
      description,
    }
  }
}

impl PartialEq for UnionDefinition {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.types == other.types
  }
}
