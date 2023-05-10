use crate::config;

/// A Wick types manifest to import types from.
#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]
pub struct TypesComponent {
  /// The OCI reference/local path of the types manifest.
  pub(crate) reference: config::AssetReference,
  /// The types to import.
  #[asset(skip)]
  pub(crate) types: Vec<String>,
}
