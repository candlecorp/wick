#![allow(missing_docs)] // delete when we move away from the `property` crate.

use std::borrow::Cow;

use wick_interface_types::Field;

use crate::config::components::OperationConfig;

#[derive(Debug, derive_builder::Builder, Clone, property::Property, serde::Serialize)]
#[property(get(disable), set(private), mut(disable))]
#[builder(setter(into))]
/// The generic definition of an Operation without any implementation details.
pub struct OperationDefinition {
  /// The name of the schematic.
  pub(crate) name: String,

  /// Any configuration required for the component to operate.
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) config: Vec<Field>,

  /// A list of the input types for the operation.
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<Field>,

  /// A list of the input types for the operation.
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<Field>,
}

impl OperationConfig for OperationDefinition {
  fn name(&self) -> &str {
    &self.name
  }

  fn inputs(&self) -> Cow<Vec<Field>> {
    Cow::Borrowed(&self.inputs)
  }

  fn outputs(&self) -> Cow<Vec<Field>> {
    Cow::Borrowed(&self.outputs)
  }
}
