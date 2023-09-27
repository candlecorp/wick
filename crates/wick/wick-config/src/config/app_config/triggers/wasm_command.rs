use std::collections::HashMap;
use std::path::Path;

use wick_asset_reference::AssetReference;
use wick_packet::RuntimeConfig;

use crate::config::template_config::Renderable;
use crate::config::{Binding, ExposedVolume, ImportDefinition};
use crate::error::ManifestError;
use crate::ExpandImports;

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
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]
/// A WebAssembly command component trigger.
pub struct WasmCommandConfig {
  /// The location of the component.
  pub(crate) reference: AssetReference,

  /// The volumes to expose to the WebAssembly component.
  #[asset(skip)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) volumes: Vec<ExposedVolume>,
}

impl ExpandImports for WasmCommandConfig {
  type Error = ManifestError;
  fn expand_imports(
    &mut self,
    _bindings: &mut Vec<Binding<ImportDefinition>>,
    _trigger_index: usize,
  ) -> Result<(), Self::Error> {
    Ok(())
  }
}

impl Renderable for WasmCommandConfig {
  fn render_config(
    &mut self,
    _source: Option<&Path>,
    _root_config: Option<&RuntimeConfig>,
    _env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    Ok(())
  }
}
