#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::collections::HashMap;

use wick_interface_types::OperationSignatures;

use crate::config::{self, LiquidJsonConfig};

/// A separate Wick manifest to use as a collection.
#[derive(
  Debug,
  Clone,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
  derive_builder::Builder,
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
  /// If applicable, the size of the send/receive buffer to allocate to the component.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) max_packet_size: Option<u32>,
}

impl OperationSignatures for ManifestComponent {
  fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    let mut config = {
      let lock = crate::config::cache::CONFIG_CACHE.lock();
      lock.get(&self.reference).cloned().map_or_else(
        || {
          panic!(
            "Can't query configuration for manifest {:?} until it has been fetched and cached",
            self.reference
          )
        },
        |c| c,
      )
    };

    let runtime_config = self.config.as_ref().and_then(|c| c.value());
    config.set_root_config(runtime_config.cloned());
    let config = match config.finish() {
      Ok(c) => c,
      Err(e) => panic!(
        "Configuration can't be rendered for manifest {:?}: {}",
        self.reference, e
      ),
    };
    let Ok(config) = config.try_component_config() else {
      panic!("Only component configurations have operation signatures");
    };

    config.component().operation_signatures()
  }
}
