use std::collections::HashMap;
use std::path::Path;

use wick_packet::RuntimeConfig;

use super::WickRouter;
use crate::config::template_config::Renderable;
use crate::config::{self, ComponentOperationExpression, ImportBinding};
use crate::error::ManifestError;

#[derive(
  Debug,
  Clone,
  derive_builder::Builder,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
)]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]
#[asset(asset(config::AssetReference))]
/// Request and response operations that run before and after the main operation.
pub struct Middleware {
  /// The middleware to apply to requests.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) request: Vec<ComponentOperationExpression>,
  /// The middleware to apply to responses.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) response: Vec<ComponentOperationExpression>,
}

impl Renderable for Middleware {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.request.render_config(source, root_config, env)?;
    self.response.render_config(source, root_config, env)
  }
}

pub(super) fn expand_for_middleware_components(
  trigger_index: usize,
  router_index: usize,
  router: &mut impl WickRouter,
  bindings: &mut Vec<ImportBinding>,
) -> Result<(), ManifestError> {
  if let Some(middleware) = router.middleware_mut() {
    for (i, operation) in middleware.request_mut().iter_mut().enumerate() {
      let id = format!(
        "trigger_{}_router_{}_request_middleware_{}",
        trigger_index, router_index, i
      );
      operation.maybe_import(&id, bindings);
    }
    for (i, operation) in middleware.response_mut().iter_mut().enumerate() {
      let id = format!(
        "trigger_{}_router_{}_response_middleware_{}",
        trigger_index, router_index, i
      );
      operation.maybe_import(&id, bindings);
    }
  }

  Ok(())
}
