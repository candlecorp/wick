use std::collections::HashMap;

use crate::config::{self, OperationSignature};

#[derive(Debug, Clone, derive_asset_container::AssetManager, Builder, property::Property)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[builder(setter(into))]
#[asset(asset(config::AssetReference))]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct WasmComponentImplementation {
  /// The location of the component.
  pub(crate) reference: config::AssetReference,

  /// The configuration for the component.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) config: Vec<wick_interface_types::Field>,

  /// The operations defined by the component.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) operations: HashMap<String, OperationSignature>,
}

impl WasmComponentImplementation {
  /// Get the signature of the component as defined by the manifest.
  #[must_use]
  pub fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.values().cloned().map(Into::into).collect()
  }
}
