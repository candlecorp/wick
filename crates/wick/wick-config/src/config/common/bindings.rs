#![allow(missing_docs)] // delete when we move away from the `property` crate.

use std::collections::HashMap;
use std::path::Path;

use asset_container::AssetManager;
use wick_asset_reference::AssetReference;
use wick_packet::RuntimeConfig;

use super::template_config::Renderable;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
/// A binding between an identifier and a target.
pub struct Binding<T>
where
  T: serde::Serialize,
{
  /// The namespace to reference the collection's components on.
  pub(crate) id: String,
  /// The kind/type of the collection.
  pub(crate) kind: T,
}

impl<T> Renderable for Binding<T>
where
  T: serde::Serialize + Renderable,
{
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), crate::error::ManifestError> {
    self.kind.render_config(source, root_config, env)
  }
}

impl<T> AssetManager for Binding<T>
where
  T: serde::Serialize + AssetManager<Asset = AssetReference>,
{
  type Asset = AssetReference;

  fn assets(&self) -> asset_container::Assets<Self::Asset> {
    self.kind.assets()
  }

  fn set_baseurl(&self, baseurl: &Path) {
    self.kind.set_baseurl(baseurl);
  }
}

impl<T> Binding<T>
where
  T: serde::Serialize,
{
  /// Create a new [Binding<ImportDefinition>] with specified name and [ImportDefinition].
  pub fn new<K: Into<String>, INTO: Into<T>>(name: K, kind: INTO) -> Self {
    Self {
      id: name.into(),
      kind: kind.into(),
    }
  }

  /// Get the ID for the binding.
  #[must_use]
  pub fn id(&self) -> &str {
    &self.id
  }

  /// Get the kind for the binding.
  #[must_use]
  pub const fn kind(&self) -> &T {
    &self.kind
  }
}
