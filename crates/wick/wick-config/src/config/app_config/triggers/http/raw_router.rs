use wick_asset_reference::AssetReference;

use super::index_to_router_id;
use super::middleware::expand_for_middleware_components;
use crate::config::{self, ComponentOperationExpression, ImportBinding};
use crate::error::ManifestError;

#[derive(Debug, Clone, derive_asset_container::AssetManager, property::Property, serde::Serialize)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]
#[must_use]
pub struct RawRouterConfig {
  #[asset(skip)]
  #[property(get(disable))]
  pub(crate) path: String,
  /// Middleware operations for this router.
  #[property(get(disable), mut(disable))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) middleware: Option<super::middleware::Middleware>,
  #[asset(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) codec: Option<config::common::Codec>,
  pub(crate) operation: ComponentOperationExpression,
}

impl super::WickRouter for RawRouterConfig {
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
  router_config: &mut RawRouterConfig,
  bindings: &mut Vec<ImportBinding>,
) -> Result<(), ManifestError> {
  expand_for_middleware_components(trigger_index, index, router_config, bindings)?;

  router_config
    .operation_mut()
    .maybe_import(&index_to_router_id(trigger_index, index), bindings);

  let router_component = config::ComponentDefinition::Native(config::components::NativeComponent {});
  let router_binding = config::ImportBinding::component(index_to_router_id(trigger_index, index), router_component);
  bindings.push(router_binding);

  Ok(())
}
