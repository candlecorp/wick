use wick_asset_reference::AssetReference;

use super::OperationInputConfig;
use crate::config::{ComponentDefinition, ComponentOperationExpression};

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
/// Normalized representation of a Time trigger configuration.
pub struct TimeTriggerConfig {
  pub(crate) schedule: ScheduleConfig,
  pub(crate) operation: ComponentOperationExpression,
  #[asset(skip)]
  pub(crate) payload: Vec<OperationInputConfig>,
}

impl TimeTriggerConfig {
  /// Returns the component id for the Time trigger.
  pub fn component(&self) -> &ComponentDefinition {
    &self.operation.component
  }

  /// Returns the payload for the Time trigger.
  #[must_use]
  pub fn payload(&self) -> &Vec<OperationInputConfig> {
    &self.payload
  }

  /// Returns the operation name for the Time trigger.
  #[must_use]
  pub fn operation(&self) -> &str {
    &self.operation.operation
  }
  pub fn schedule(&self) -> &ScheduleConfig {
    &self.schedule
  }
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
#[must_use]
pub struct ScheduleConfig {
  #[asset(skip)]
  pub(crate) cron: String,
  #[asset(skip)]
  pub(crate) repeat: u16,
}

impl ScheduleConfig {
  #[must_use]
  pub fn cron(&self) -> &String {
    &self.cron
  }

  #[must_use]
  pub fn repeat(&self) -> u16 {
    self.repeat
  }
}
