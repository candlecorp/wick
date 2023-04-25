use std::collections::HashMap;

use wick_interface_types::Field;

use crate::config;

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(config::AssetReference)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct WasmComponentImplementation {
  /// The location of the component.
  pub(crate) reference: config::AssetReference,

  #[asset(skip)]
  pub(crate) operations: HashMap<String, OperationSignature>,
}

impl WasmComponentImplementation {
  /// Get the signature of the component as defined by the manifest.
  #[must_use]
  pub fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.values().cloned().map(Into::into).collect()
  }
}

#[derive(Debug, Clone)]
pub struct OperationSignature {
  /// The name of the schematic.
  pub name: String,

  /// A list of the input types for the operation.
  pub inputs: Vec<Field>,

  /// A list of the input types for the operation.
  pub outputs: Vec<Field>,
}

impl WasmComponentImplementation {
  /// Get the operations implemented by this component.
  #[must_use]
  pub fn operations(&self) -> &HashMap<String, OperationSignature> {
    &self.operations
  }

  /// Get the reference location to the component.
  pub fn reference(&self) -> &config::AssetReference {
    &self.reference
  }
}
