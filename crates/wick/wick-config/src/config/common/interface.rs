use std::collections::HashMap;

use wick_interface_types::TypeDefinition;

use crate::component_config::OperationSignature;

#[derive(Debug, Default, Clone, derive_assets::AssetManager)]
#[asset(crate::config::AssetReference)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct InterfaceDefinition {
  /// Types used by the component's operations.
  #[asset(skip)]
  pub(crate) types: Vec<TypeDefinition>,

  /// The operations the component exposes.
  #[asset(skip)]
  pub(crate) operations: HashMap<String, OperationSignature>,
}
