use serde::{Deserialize, Serialize};

use crate::{contents_equal, Field, OperationSignature, TypeDefinition};

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
  pub config: Vec<Field>,
}

impl PartialEq for ComponentSignature {
  fn eq(&self, other: &Self) -> bool {
    let types_equal = contents_equal(&self.types, &other.types);
    let operations_equal = contents_equal(&self.operations, &other.operations);
    let config_equal = contents_equal(&self.config, &other.config);
    let wellknown_equal = contents_equal(&self.wellknown, &other.wellknown);

    self.format == other.format && types_equal && operations_equal && config_equal && wellknown_equal
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
  pub fn get_operation(&self, operation_name: &str) -> Option<&OperationSignature> {
    self.operations.iter().find(|op| op.name == operation_name)
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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
}

impl ComponentMetadata {
  pub fn new(version: Option<impl AsRef<str>>) -> Self {
    Self {
      version: version.map(|v| v.as_ref().to_owned()),
    }
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
