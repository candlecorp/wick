#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::collections::HashMap;

use crate::config::{self, LiquidJsonConfig};

/// A separate Wick manifest to use as a collection.
#[derive(
  Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property, serde::Serialize, Builder,
)]
#[builder(setter(into))]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]
/// A Wick manifest to use as a component.
pub struct ManifestComponent {
  /// The OCI reference/local path of the manifest to use as a component.
  pub(crate) reference: config::AssetReference,
  /// The configuration for the component.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) config: Option<LiquidJsonConfig>,
  /// The components to provide to the referenced component.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub(crate) provide: HashMap<String, String>,
}
