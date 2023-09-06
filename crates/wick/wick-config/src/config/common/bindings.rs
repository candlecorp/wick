#![allow(missing_docs)] // delete when we move away from the `property` crate.

use std::collections::HashMap;
use std::path::Path;

use wick_packet::RuntimeConfig;

use super::template_config::Renderable;
use super::{ComponentDefinition, HighLevelComponent, ImportDefinition, InterfaceDefinition};
use crate::config::components::WasmComponent;
use crate::config::{self};

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property, serde::Serialize)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(config::AssetReference))]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct ImportBinding {
  /// The namespace to reference the collection's components on.
  #[asset(skip)]
  pub(crate) id: String,
  /// The kind/type of the collection.
  pub(crate) kind: ImportDefinition,
}

impl Renderable for ImportBinding {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), crate::error::ManifestError> {
    self.kind.render_config(source, root_config, env)
  }
}

impl ImportBinding {
  /// Create a new [ImportBinding] with specified name and [ImportDefinition].
  pub fn new<T: AsRef<str>>(name: T, kind: ImportDefinition) -> Self {
    Self {
      id: name.as_ref().to_owned(),
      kind,
    }
  }

  /// Get the configuration object for the collection.
  #[must_use]
  pub fn config(&self) -> Option<&RuntimeConfig> {
    match &self.kind {
      ImportDefinition::Component(c) => c.config().and_then(|v| v.value()),
      ImportDefinition::Types(_) => None,
    }
  }

  /// Get the configuration object for the collection.
  #[must_use]
  pub fn provide(&self) -> Option<&HashMap<String, String>> {
    match &self.kind {
      ImportDefinition::Component(c) => c.provide(),
      ImportDefinition::Types(_) => None,
    }
  }

  /// Initialize a new import for the specified [ComponentDefinition].
  pub fn component<T: AsRef<str>>(name: T, component: ComponentDefinition) -> Self {
    #[allow(deprecated)]
    Self::new(name, ImportDefinition::Component(component))
  }

  /// Create a new Wasm component definition.
  pub fn wasm<T: AsRef<str>>(name: T, component: WasmComponent) -> Self {
    #[allow(deprecated)]
    Self::new(name, ImportDefinition::Component(ComponentDefinition::Wasm(component)))
  }

  /// Create a new GrpcUrl component definition.
  pub fn grpc_url<T: AsRef<str>>(name: T, component: config::components::GrpcUrlComponent) -> Self {
    Self::new(
      name,
      ImportDefinition::Component(ComponentDefinition::GrpcUrl(component)),
    )
  }

  /// Create a new Manifest component definition.
  pub fn manifest<T: AsRef<str>>(name: T, component: config::components::ManifestComponent) -> Self {
    Self::new(
      name,
      ImportDefinition::Component(ComponentDefinition::Manifest(component)),
    )
  }

  /// Create a new High level component definition.
  pub fn high_level<T: AsRef<str>>(name: T, component: HighLevelComponent) -> Self {
    Self::new(
      name,
      ImportDefinition::Component(ComponentDefinition::HighLevelComponent(component)),
    )
  }
}

#[derive(Debug, Default, Clone, derive_asset_container::AssetManager, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(crate::config::AssetReference))]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct BoundInterface {
  /// The namespace to reference the collection's components on.
  #[asset(skip)]
  pub(crate) id: String,
  /// The kind/type of the collection.
  pub(crate) kind: InterfaceDefinition,
}
