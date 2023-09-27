use std::collections::HashMap;
use std::path::Path;

use wick_asset_reference::AssetReference;
use wick_packet::RuntimeConfig;

use super::index_to_router_id;
use super::middleware::expand_for_middleware_components;
use crate::config::bindings::BoundIdentifier;
use crate::config::template_config::Renderable;
use crate::config::{self, Binding, ImportDefinition};
use crate::error::ManifestError;

#[derive(
  Debug, Clone, derive_builder::Builder, derive_asset_container::AssetManager, property::Property, serde::Serialize,
)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]
#[must_use]
pub struct StaticRouterConfig {
  #[asset(skip)]
  #[property(get(disable))]
  pub(crate) path: String,
  /// Middleware operations for this router.
  #[property(get(disable))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) middleware: Option<super::middleware::Middleware>,
  #[asset(skip)]
  pub(crate) volume: BoundIdentifier,
  #[asset(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) fallback: Option<String>,
  #[asset(skip)]
  pub(crate) indexes: bool,
}

impl super::WickRouter for StaticRouterConfig {
  fn middleware(&self) -> Option<&super::Middleware> {
    self.middleware.as_ref()
  }

  fn middleware_mut(&mut self) -> Option<&mut super::Middleware> {
    self.middleware.as_mut()
  }

  fn path(&self) -> &str {
    &self.path
  }
}

impl Renderable for StaticRouterConfig {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.middleware.render_config(source, root_config, env)
  }
}

pub(crate) fn process_runtime_config(
  trigger_index: usize,
  index: usize,
  router_config: &mut StaticRouterConfig,
  bindings: &mut Vec<Binding<ImportDefinition>>,
) -> Result<(), ManifestError> {
  expand_for_middleware_components(trigger_index, index, router_config, bindings)?;

  let router_component = config::ComponentDefinition::Native(config::components::NativeComponent {});
  let router_binding = config::Binding::new(
    index_to_router_id(trigger_index, index),
    ImportDefinition::component(router_component),
  );
  bindings.push(router_binding);
  Ok(())
}
