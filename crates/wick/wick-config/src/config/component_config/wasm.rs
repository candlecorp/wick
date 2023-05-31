use std::collections::HashMap;

use wick_interface_types::Field;

use crate::config::{self, OperationSignature};

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(config::AssetReference))]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct WasmComponentImplementation {
  /// The configuration for the component.
  #[asset(skip)]
  pub(crate) config: Vec<Field>,

  /// The location of the component.
  pub(crate) reference: config::AssetReference,

  /// The operations defined by the component.
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
