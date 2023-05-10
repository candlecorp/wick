use wick_asset_reference::AssetReference;

use super::OperationInputConfig;
use crate::config::ComponentOperationExpression;

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(AssetReference))]
/// Normalized representation of a Time trigger configuration.
pub struct TimeTriggerConfig {
  pub(crate) schedule: ScheduleConfig,
  pub(crate) operation: ComponentOperationExpression,
  #[asset(skip)]
  pub(crate) payload: Vec<OperationInputConfig>,
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(AssetReference))]
#[must_use]
pub struct ScheduleConfig {
  #[asset(skip)]
  pub(crate) cron: String,
  #[asset(skip)]
  pub(crate) repeat: u16,
}
