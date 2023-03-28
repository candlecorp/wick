use std::collections::HashMap;

use wick_interface_types::{Field, TypeDefinition};

#[derive(Debug, Clone)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct WasmComponentConfiguration {
  /// The location of the component.
  pub(crate) reference: String,

  /// Types used by the component's operations.
  pub(crate) types: Vec<TypeDefinition>,

  /// The operations the component exposes.
  pub(crate) operations: HashMap<String, OperationSignature>,
}

impl WasmComponentConfiguration {}

#[derive(Debug, Clone)]
pub struct OperationSignature {
  /// The name of the schematic.
  pub name: String,

  /// A list of the input types for the operation.
  pub inputs: Vec<Field>,

  /// A list of the input types for the operation.
  pub outputs: Vec<Field>,
}
