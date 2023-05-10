use std::collections::HashMap;

use config::common::OperationSignature;
use wick_interface_types::TypeDefinition;

use crate::config;

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
  pub(crate) operations: HashMap<String, OperationSignature>,
}
