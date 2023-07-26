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
  /// The components to provide to the referenced component.
  #[asset(skip)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub(crate) provide: HashMap<String, String>,
}
