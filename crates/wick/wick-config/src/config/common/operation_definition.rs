use serde_json::Value;
use wick_interface_types::Field;

#[derive(Debug, Clone)]
pub struct OperationSignature {
  /// The name of the schematic.
  pub name: String,

  /// Any configuration required for the component to operate.
  pub config: Option<Value>,

  /// A list of the input types for the operation.
  pub inputs: Vec<Field>,

  /// A list of the input types for the operation.
  pub outputs: Vec<Field>,
}
