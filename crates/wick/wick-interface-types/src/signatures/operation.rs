use serde::{Deserialize, Serialize};

use crate::{contents_equal, Field, Type};

/// The signature of a Wick component, including its input and output types.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq)]
#[must_use]
pub struct OperationSignature {
  /// The name of the component.
  #[serde(default)]
  pub name: String,
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
