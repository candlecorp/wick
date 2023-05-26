use wick_interface_types::Field;

#[derive(Debug, Builder, Clone, property::Property)]
#[property(get(public), set(private), mut(disable))]
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
