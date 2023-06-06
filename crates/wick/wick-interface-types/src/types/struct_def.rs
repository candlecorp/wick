use serde::{Deserialize, Serialize};

use crate::{contents_equal, is_false, Field};

/// Signatures of struct-like type definitions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq)]
#[must_use]
pub struct StructDefinition {
  /// The name of the struct.
  pub name: String,
  /// The fields in this struct.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub fields: Vec<Field>,
  /// The optional description of the struct.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Whether this type is imported.
  #[serde(default, skip_serializing_if = "is_false")]
  pub imported: bool,
}

impl PartialEq for StructDefinition {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && contents_equal(&self.fields, &other.fields)
  }
}

impl StructDefinition {
  /// Constructor for [StructDefinition]
  pub fn new<T: AsRef<str>>(name: T, fields: Vec<Field>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      fields,
      imported: false,
      description: None,
    }
  }
}
