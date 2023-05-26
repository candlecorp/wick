use serde_json::Value;

use crate::config::*;
mod cli;
mod http;
mod time;

pub use cli::*;
pub use http::*;
pub use time::*;

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]

/// Normalized representation of a trigger definition.
pub enum TriggerDefinition {
  /// A CLI trigger.
  Cli(CliConfig),
  /// An HTTP trigger.
  Http(HttpTriggerConfig),
  /// A time trigger.
  Time(TimeTriggerConfig),
}

impl TriggerDefinition {
  /// Returns the kind of trigger.
  pub fn kind(&self) -> TriggerKind {
    match self {
      TriggerDefinition::Cli(_) => TriggerKind::Cli,
      TriggerDefinition::Http(_) => TriggerKind::Http,
      TriggerDefinition::Time(_) => TriggerKind::Time,
    }
  }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
#[must_use]
/// The kind of trigger.
pub enum TriggerKind {
  /// A CLI trigger.
  Cli,
  /// An Http trigger.
  Http,
  /// A time trigger.
  Time,
}

impl std::fmt::Display for TriggerKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TriggerKind::Cli => f.write_str("CLI"),
      TriggerKind::Http => f.write_str("HTTP"),
      TriggerKind::Time => f.write_str("TIME"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
#[must_use]

pub struct OperationInputConfig {
  #[asset(skip)]
  pub(crate) name: String,
  #[asset(skip)]
  pub(crate) value: Value,
}

impl OperationInputConfig {
  #[must_use]
  pub fn name(&self) -> &String {
    &self.name
  }

  #[must_use]
  pub fn value(&self) -> &Value {
    &self.value
  }
}
