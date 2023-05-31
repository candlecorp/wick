use std::collections::HashMap;

use crate::config;

/// A WebAssembly collection.
#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]
pub struct WasmComponent {
  /// The OCI reference/local path of the collection.
  pub(crate) reference: config::AssetReference,
  /// The configuration for the collection
  #[asset(skip)]
  pub(crate) config: Option<wick_packet::GenericConfig>,
  /// Permissions for this collection
  #[asset(skip)]
  pub(crate) permissions: Permissions,
  /// The components to provide to the referenced component.
  #[asset(skip)]
  pub(crate) provide: HashMap<String, String>,
}

/// The permissions object for a collection
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize, property::Property)]
#[property(get(public), set(private), mut(disable))]
pub struct Permissions {
  /// A map of directories (Note: TO -> FROM) to expose to the collection.
  #[serde(default)]
  pub(crate) dirs: HashMap<String, String>,
}

impl Permissions {
  /// Create a new permissions object
  #[must_use]
  pub fn new(dirs: HashMap<String, String>) -> Self {
    Self { dirs }
  }
}
