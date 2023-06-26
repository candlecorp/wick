#![allow(missing_docs)] // delete when we move away from the `property` crate.

use wick_interface_types::Field;

#[derive(Debug, Builder, Clone, property::Property)]
#[property(get(disable), set(private), mut(disable))]
#[builder(setter(into))]
/// The generic definition of an Operation without any implementation details.
pub struct OperationDefinition {
  /// The name of the schematic.
  pub(crate) name: String,

  /// Any configuration required for the component to operate.
  #[builder(default)]
  pub(crate) config: Vec<Field>,

  /// A list of the input types for the operation.
  #[builder(default)]
  pub(crate) inputs: Vec<Field>,

  /// A list of the input types for the operation.
  #[builder(default)]
  pub(crate) outputs: Vec<Field>,
}
