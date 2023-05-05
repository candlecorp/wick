use serde::{Deserialize, Serialize};

use crate::{contents_equal, is_false, Field};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

impl TypeDefinition {
  /// Get the name of the type.
  #[must_use]
  pub fn name(&self) -> &str {
    match self {
      TypeDefinition::Struct(s) => &s.name,
      TypeDefinition::Enum(e) => &e.name,
    }
  }
}

/// Signatures of enum type definitions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq)]
#[must_use]
pub struct EnumSignature {
  /// The name of the enum.
  pub name: String,
  /// The variants in the enum.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub variants: Vec<EnumVariant>,
  /// The optional description of the enum.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Whether this type is imported.
  #[serde(default, skip_serializing_if = "is_false")]
  pub imported: bool,
}

impl EnumSignature {
  /// Constructor for [EnumSignature]
  pub fn new<T: AsRef<str>>(name: T, variants: Vec<EnumVariant>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      variants,
      imported: false,
      description: None,
    }
  }
}

impl PartialEq for EnumSignature {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.variants == other.variants
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[must_use]
/// An enum variant definition
pub struct EnumVariant {
  /// The name of the variant.
  pub name: String,
  /// The index of the variant.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub index: Option<u32>,
  /// The optional value of the variant.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub value: Option<String>,
  /// The optional description of the variant.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

impl EnumVariant {
  /// Constructor for [EnumVariant]
  pub fn new<T: AsRef<str>>(name: T, index: Option<u32>, value: Option<String>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      index,
      value,
      description: None,
    }
  }
}

/// Signatures of struct-like type definitions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq)]
#[must_use]
pub struct StructSignature {
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

impl PartialEq for StructSignature {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && contents_equal(&self.fields, &other.fields)
  }
}

impl StructSignature {
  /// Constructor for [StructSignature]
  pub fn new<T: AsRef<str>>(name: T, fields: Vec<Field>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      fields,
      imported: false,
      description: None,
    }
  }
}
