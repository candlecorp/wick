#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::collections::HashMap;

use crate::config;

/// A separate Wick manifest to use as a collection.
#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]
/// A Wick manifest to use as a component.
pub struct ManifestComponent {
  /// The OCI reference/local path of the manifest to use as a component.
  pub(crate) reference: config::AssetReference,
  /// The configuration for the component.
  #[asset(skip)]
  pub(crate) config: Option<wick_packet::GenericConfig>,
  /// The components to provide to the referenced component.
  #[asset(skip)]
  pub(crate) provide: HashMap<String, String>,
}
