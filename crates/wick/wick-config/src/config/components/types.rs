use asset_container::Asset;

use crate::config;

/// A Wick types manifest to import types from.
#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(config::AssetReference)]
pub struct TypesComponent {
  /// The OCI reference/local path of the types manifest.
  pub reference: config::AssetReference,
  /// The types to import.
  #[asset(skip)]
  pub types: Vec<String>,
}
