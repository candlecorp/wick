use wick_asset_reference::AssetReference;

use crate::config::{self, ComponentOperationExpression};

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
#[must_use]
pub struct RawRouterConfig {
  #[asset(skip)]
  pub(crate) path: String,
  #[asset(skip)]
  pub(crate) codec: Option<config::components::Codec>,
  pub(crate) operation: ComponentOperationExpression,
}

impl RawRouterConfig {
  #[must_use]
  pub fn path(&self) -> &str {
    &self.path
  }

  #[must_use]
  pub fn operation(&self) -> &ComponentOperationExpression {
    &self.operation
  }

  #[must_use]
  pub fn codec(&self) -> &Option<config::components::Codec> {
    &self.codec
  }
}
