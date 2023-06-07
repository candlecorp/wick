use wick_interface_types::Field;

#[derive(Debug, Builder, Clone, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[builder(setter(into))]
pub struct OperationSignature {
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
