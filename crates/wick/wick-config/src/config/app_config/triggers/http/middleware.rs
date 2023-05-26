use crate::config::{self, ComponentOperationExpression};

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]
/// Request and response operations that run before and after the main operation.
pub struct Middleware {
  /// The middleware to apply to requests.
  pub(crate) request: Vec<ComponentOperationExpression>,
  /// The middleware to apply to responses.
  pub(crate) response: Vec<ComponentOperationExpression>,
}
