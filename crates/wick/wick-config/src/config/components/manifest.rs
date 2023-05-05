use std::collections::HashMap;

use crate::config;

/// A separate Wick manifest to use as a collection.
#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(config::AssetReference))]
pub struct ManifestComponent {
  /// The OCI reference/local path of the manifest to use as a collection.
  pub reference: config::AssetReference,
  /// The configuration for the collection
  #[asset(skip)]
  pub config: Option<wick_packet::OperationConfig>,
  /// The components to provide to the referenced component.
  #[asset(skip)]
  pub provide: HashMap<String, String>,
}
