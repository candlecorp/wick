#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::collections::HashMap;

use crate::config::{self, LiquidJsonConfig};

/// A WebAssembly collection.
#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]
pub struct WasmComponent {
  /// The OCI reference/local path of the collection.
  pub(crate) reference: config::AssetReference,
  /// The configuration for the collection
  #[asset(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) config: Option<LiquidJsonConfig>,
  /// Permissions for this collection
  #[asset(skip)]
  pub(crate) permissions: Permissions,
  /// The components to provide to the referenced component.
  #[asset(skip)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub(crate) provide: HashMap<String, String>,
}

/// The permissions object for a collection
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize, property::Property)]
#[property(get(public), set(private), mut(disable))]
pub struct Permissions {
  /// A map of directories (Note: TO -> FROM) to expose to the collection.
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub(crate) dirs: HashMap<String, String>,
}

impl Permissions {
  /// Create a new permissions object
  #[must_use]
  pub fn new(dirs: HashMap<String, String>) -> Self {
    Self { dirs }
  }
}
