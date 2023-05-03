use wick_interface_types::Field;

#[derive(Debug, Builder, Clone)]
pub struct OperationSignature {
  /// The name of the schematic.
  pub(crate) name: String,

  /// Any configuration required for the component to operate.
  pub(crate) config: Vec<Field>,

  /// A list of the input types for the operation.
  pub(crate) inputs: Vec<Field>,

  /// A list of the input types for the operation.
  pub(crate) outputs: Vec<Field>,
}

impl OperationSignature {
  #[must_use]
  /// Get the name of the operation.
  pub fn name(&self) -> &str {
    &self.name
  }

  #[must_use]
  /// Get the inputs of the operation.
  pub fn inputs(&self) -> &[Field] {
    &self.inputs
  }

  #[must_use]
  /// Get the outputs of the operation.
  pub fn outputs(&self) -> &[Field] {
    &self.outputs
  }

  #[must_use]
  /// Get the configuration of the operation.
  pub fn config(&self) -> &[Field] {
    &self.config
  }
}
