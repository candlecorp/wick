use serde::{Deserialize, Serialize};

use crate::{is_false, Type};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct Field {
  /// The name of the field.
  pub name: String,

  /// The type of the field.
  #[serde(rename = "type")]
  #[cfg_attr(feature = "parser", serde(deserialize_with = "crate::types::deserialize_type"))]
  #[cfg_attr(
    feature = "yaml",
    serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
  )]
  pub ty: Type,

  /// Whether the field is required.
  #[cfg(feature = "value")]
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
  pub fn new<T: Into<String>>(name: T, ty: Type) -> Self {
    Self::new_with_description(name, ty, None)
  }

  pub fn new_with_description<T: Into<String>>(name: T, ty: Type, desc: Option<String>) -> Self {
    Self {
      name: name.into(),
      description: desc,
      #[cfg(feature = "value")]
      default: None,
      required: !matches!(ty, Type::Optional { .. }),
      ty,
    }
  }

  /// Get the name of the field
  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Get the type of the field
  pub const fn ty(&self) -> &Type {
    &self.ty
  }

  /// Get the description of the field
  #[must_use]
  pub fn description(&self) -> Option<&str> {
    self.description.as_deref()
  }

  /// Get the default value of the field
  #[must_use]
  #[cfg(feature = "value")]
  pub const fn default(&self) -> Option<&serde_json::Value> {
    self.default.as_ref()
  }

  /// Get whether the field is required
  #[must_use]
  pub const fn required(&self) -> bool {
    self.required
  }

  /// Consume the [Field] and return a [FieldValue] with the given value.
  #[must_use]
  #[cfg(feature = "value")]
  pub fn with_value(self, value: impl Into<serde_json::Value>) -> FieldValue {
    FieldValue::new(self, value.into())
  }
}

impl std::fmt::Display for Field {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.name)?;
    f.write_str(": ")?;
    self.ty.fmt(f)
  }
}

/// A field and its value.
#[cfg(feature = "value")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct FieldValue {
  /// The field.
  pub field: Field,
  /// The value of the field.
  pub value: serde_json::Value,
}

#[cfg(feature = "value")]
impl FieldValue {
  /// Create a new field value.
  #[must_use]
  pub const fn new(field: Field, value: serde_json::Value) -> Self {
    Self { field, value }
  }

  /// Get the name of the field
  #[must_use]
  pub fn name(&self) -> &str {
    &self.field.name
  }

  /// Get the type of the field
  pub const fn signature(&self) -> &Type {
    &self.field.ty
  }

  /// Get the value of the field
  #[must_use]
  pub const fn value(&self) -> &serde_json::Value {
    &self.value
  }
}
