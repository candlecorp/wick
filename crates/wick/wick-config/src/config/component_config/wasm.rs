use wick_interface_types::OperationSignatures;

use crate::config::components::{ComponentConfig, OperationConfig};
use crate::config::{self, OperationDefinition};
use crate::utils::VecMapInto;

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
  #[property(skip)]
  pub(crate) operations: Vec<OperationDefinition>,
}

impl OperationSignatures for WasmComponentImplementation {
  fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.clone().map_into()
  }
}

impl ComponentConfig for WasmComponentImplementation {
  type Operation = OperationDefinition;

  fn operations(&self) -> &[Self::Operation] {
    &self.operations
  }

  fn operations_mut(&mut self) -> &mut Vec<Self::Operation> {
    &mut self.operations
  }
}

impl OperationConfig for OperationDefinition {
  fn name(&self) -> &str {
    &self.name
  }
}
