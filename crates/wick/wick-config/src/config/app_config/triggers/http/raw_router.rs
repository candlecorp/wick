use wick_asset_reference::AssetReference;

use crate::config::{self, ComponentOperationExpression};

#[derive(Debug, Clone, derive_asset_container::AssetManager, property::Property, serde::Serialize)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]
#[must_use]
pub struct RawRouterConfig {
  #[asset(skip)]
  #[property(get(disable))]
  pub(crate) path: String,
  /// Middleware operations for this router.
  #[property(get(disable))]
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

  fn path(&self) -> &str {
    &self.path
  }
}
