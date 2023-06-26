#![allow(missing_docs)] // delete when we move away from the `property` crate.

use config::common::OperationDefinition;
use wick_interface_types::{OperationSignatures, TypeDefinition};

use crate::config::components::ComponentConfig;
use crate::config::{self};

#[derive(Debug, Default, Clone, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(crate::config::AssetReference))]
#[must_use]
/// The Wick representation of an interface.
pub struct InterfaceDefinition {
  /// Types used by the interface's operations.
  #[asset(skip)]
  pub(crate) types: Vec<TypeDefinition>,

  /// The operations the interface exposes.
  #[asset(skip)]
  #[property(skip)]
  pub(crate) operations: Vec<OperationDefinition>,
}

impl ComponentConfig for InterfaceDefinition {
  type Operation = OperationDefinition;

  fn operations(&self) -> &[Self::Operation] {
    &self.operations
  }

  fn operations_mut(&mut self) -> &mut Vec<Self::Operation> {
    &mut self.operations
  }
}

impl OperationSignatures for InterfaceDefinition {
  fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.iter().cloned().map(Into::into).collect()
  }
}
