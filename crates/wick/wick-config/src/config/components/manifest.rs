use std::collections::HashMap;

use crate::config;

/// A separate Wick manifest to use as a collection.
#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]
pub struct ManifestComponent {
  /// The OCI reference/local path of the manifest to use as a collection.
  pub(crate) reference: config::AssetReference,
  /// The configuration for the collection
  #[asset(skip)]
  pub(crate) config: Option<wick_packet::OperationConfig>,
  /// The components to provide to the referenced component.
  #[asset(skip)]
  pub(crate) provide: HashMap<String, String>,
}
