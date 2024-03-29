#![allow(missing_docs)] // delete when we move away from the `property` crate.
use crate::config;

/// A Wick types manifest to import types from.
#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]
pub struct TypesComponent {
  /// The OCI reference/local path of the types manifest.
  pub(crate) reference: config::AssetReference,
  /// The types to import.
  #[asset(skip)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<String>,
}
