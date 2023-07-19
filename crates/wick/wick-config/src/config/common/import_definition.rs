use std::collections::HashMap;

use wick_packet::RuntimeConfig;

use crate::config::{self};
use crate::error::ManifestError;

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, serde::Serialize)]
#[asset(asset(config::AssetReference))]
/// The kinds of collections that can operate in a flow.
#[must_use]
#[serde(rename_all = "kebab-case")]
pub enum ImportDefinition {
  /// A wick component.
  Component(config::ComponentDefinition),
  /// A type manifest.
  Types(config::components::TypesComponent),
}

impl ImportDefinition {
  /// Returns true if the definition is a reference to another component.
  #[must_use]
  pub fn is_reference(&self) -> bool {
    if let ImportDefinition::Component(c) = self {
      return c.is_reference();
    }
    false
  }

  /// Get the configuration associated with this import.
  #[must_use]
  pub fn config(&self) -> Option<&RuntimeConfig> {
    match self {
      ImportDefinition::Component(v) => v.config().and_then(|v| v.value()),
      ImportDefinition::Types(_) => None,
    }
  }

  /// Render the configuration associated with this import.
  pub(crate) fn render_config(
    &mut self,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    match self {
      ImportDefinition::Component(v) => v.render_config(root_config, env),
      ImportDefinition::Types(_) => Ok(()),
    }
  }
}
