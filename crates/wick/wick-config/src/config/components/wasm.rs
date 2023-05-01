use std::collections::HashMap;

use asset_container::Asset;
use serde_json::Value;

use crate::config;

/// A WebAssembly collection.
#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(config::AssetReference)]
pub struct WasmComponent {
  /// The OCI reference/local path of the collection.
  pub reference: config::AssetReference,
  /// The configuration for the collection
  #[asset(skip)]
  pub config: Value,
  /// Permissions for this collection
  #[asset(skip)]
  pub permissions: Permissions,
  /// The components to provide to the referenced component.
  #[asset(skip)]
  pub provide: HashMap<String, String>,
}

/// The permissions object for a collection
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Permissions {
  /// A map of directories (Note: TO -> FROM) to expose to the collection.
  #[serde(default)]
  pub dirs: HashMap<String, String>,
}
