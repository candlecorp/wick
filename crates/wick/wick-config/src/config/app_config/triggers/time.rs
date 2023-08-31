use std::collections::HashMap;
use std::path::Path;

use wick_asset_reference::AssetReference;
use wick_packet::RuntimeConfig;

use super::OperationInputConfig;
use crate::config::template_config::Renderable;
use crate::config::{ComponentOperationExpression, ImportBinding};
use crate::error::ManifestError;
use crate::ExpandImports;

#[derive(
  Debug,
  Clone,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
  derive_builder::Builder,
)]
#[builder(setter(into))]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]
#[asset(asset(AssetReference))]
/// Normalized representation of a Time trigger configuration.
pub struct TimeTriggerConfig {
  pub(crate) schedule: ScheduleConfig,
  pub(crate) operation: ComponentOperationExpression,
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) payload: Vec<OperationInputConfig>,
}

impl Renderable for TimeTriggerConfig {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.operation.render_config(source, root_config, env)
  }
}

#[derive(
  Debug,
  Clone,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
  derive_builder::Builder,
)]
#[builder(setter(into))]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(AssetReference))]
#[must_use]
pub struct ScheduleConfig {
  #[asset(skip)]
  pub(crate) cron: String,
  #[asset(skip)]
  #[builder(default)]
  pub(crate) repeat: u16,
}

impl ExpandImports for TimeTriggerConfig {
  type Error = ManifestError;
  fn expand_imports(&mut self, bindings: &mut Vec<ImportBinding>, trigger_index: usize) -> Result<(), Self::Error> {
    let id = format!("trigger_{}", trigger_index);
    self.operation_mut().maybe_import(&id, bindings);
    Ok(())
  }
}
