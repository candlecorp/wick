use serde_json::Value;

use super::{ComponentDefinition, InterfaceDefinition};
use crate::config::{self};

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(config::AssetReference)]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct BoundComponent {
  /// The namespace to reference the collection's components on.
  #[asset(skip)]
  pub id: String,
  /// The kind/type of the collection.
  pub kind: ComponentDefinition,
}

impl BoundComponent {
  /// Create a new [CollectionDefinition] with specified name and type.
  pub fn new(name: impl AsRef<str>, kind: ComponentDefinition) -> Self {
    Self {
      id: name.as_ref().to_owned(),
      kind,
    }
  }

  /// Get the configuration object for the collection.
  #[must_use]
  pub fn config(&self) -> Option<&Value> {
    match &self.kind {
      ComponentDefinition::Native(_) => None,
      #[allow(deprecated)]
      ComponentDefinition::Wasm(v) => Some(&v.config),
      ComponentDefinition::GrpcUrl(v) => Some(&v.config),
      ComponentDefinition::Manifest(v) => Some(&v.config),
      ComponentDefinition::HighLevelComponent(_) => None,
      ComponentDefinition::Reference(_) => None,
    }
  }
}

#[derive(Debug, Default, Clone, derive_asset_container::AssetManager)]
#[asset(crate::config::AssetReference)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct BoundInterface {
  /// The namespace to reference the collection's components on.
  #[asset(skip)]
  pub id: String,
  /// The kind/type of the collection.
  pub kind: InterfaceDefinition,
}
