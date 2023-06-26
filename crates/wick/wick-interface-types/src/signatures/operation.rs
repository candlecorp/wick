use serde::{Deserialize, Serialize};

use crate::{contents_equal, Field, Type};

/// The signature of a Wick component, including its input and output types.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq)]
#[must_use]
pub struct OperationSignature {
  /// The name of the component.
  #[serde(default)]
  pub name: String,

  /// The operation's configuration.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub config: Vec<Field>,

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
  /// Get the name of the operation.
  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Get the operation's configuration.
  #[must_use]
  pub fn config(&self) -> &[Field] {
    &self.config
  }

  /// Get the operation's inputs.
  #[must_use]
  pub fn inputs(&self) -> &[Field] {
    &self.inputs
  }

  /// Get the operation's outputs.
  #[must_use]
  pub fn outputs(&self) -> &[Field] {
    &self.outputs
  }

  /// Create a new [OperationSignature] with the passed name.
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      ..Default::default()
    }
  }

  /// Add an input port.
  pub fn add_input(mut self, name: impl AsRef<str>, ty: Type) -> Self {
    self.inputs.push(Field::new(name, ty));
    self
  }

  /// Add an input port.
  pub fn add_output(mut self, name: impl AsRef<str>, ty: Type) -> Self {
    self.outputs.push(Field::new(name, ty));
    self
  }
}
