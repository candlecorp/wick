use std::collections::HashMap;

use wick_asset_reference::AssetReference;
use wick_packet::RuntimeConfig;

use super::index_to_router_id;
use super::middleware::expand_for_middleware_components;
use crate::config::template_config::Renderable;
use crate::config::{self, ImportBinding};
use crate::error::ManifestError;

#[derive(
  Debug, Clone, derive_builder::Builder, derive_asset_container::AssetManager, property::Property, serde::Serialize,
)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]

pub struct ProxyRouterConfig {
  /// The path to start serving this router from.
  #[asset(skip)]
  #[property(get(disable))]
  pub(crate) path: String,
  /// Middleware operations for this router.
  #[property(get(disable))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) middleware: Option<super::middleware::Middleware>,
  /// The URL resource to proxy to.
  #[asset(skip)]
  pub(crate) url: String,
  /// Whether or not to strip the router's path from the proxied request.
  #[asset(skip)]
  pub(crate) strip_path: bool,
}

impl Renderable for ProxyRouterConfig {
  fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.middleware.render_config(root_config, env)
  }
}

impl super::WickRouter for ProxyRouterConfig {
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

pub(crate) fn process_runtime_config(
  trigger_index: usize,
  index: usize,
  router_config: &mut ProxyRouterConfig,
  bindings: &mut Vec<ImportBinding>,
) -> Result<(), ManifestError> {
  expand_for_middleware_components(trigger_index, index, router_config, bindings)?;
  let router_component = config::ComponentDefinition::Native(config::components::NativeComponent {});
  let router_binding = config::ImportBinding::component(index_to_router_id(trigger_index, index), router_component);
  bindings.push(router_binding);
  Ok(())
}
