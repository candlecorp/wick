use wick_asset_reference::AssetReference;

use crate::config::{self, ComponentOperationExpression};

#[derive(Debug, Clone, derive_asset_container::AssetManager, property::Property)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]
#[must_use]
pub struct RawRouterConfig {
  #[asset(skip)]
  pub(crate) path: String,
  #[asset(skip)]
  pub(crate) codec: Option<config::components::Codec>,
  pub(crate) operation: ComponentOperationExpression,
}
