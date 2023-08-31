use std::collections::HashMap;
use std::path::Path;

use wick_packet::RuntimeConfig;

use super::template_config::Renderable;
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

crate::impl_from_for!(ImportDefinition, Component, config::ComponentDefinition);
crate::impl_from_for!(ImportDefinition, Types, config::components::TypesComponent);

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

  /// Get the configuration kind for this import.
  #[must_use]
  pub fn kind(&self) -> ImportKind {
    match self {
      ImportDefinition::Component(_) => ImportKind::Component,
      ImportDefinition::Types(_) => ImportKind::Types,
    }
  }
}

/// The kind of import an [ImportDefinition] is.
#[derive(Debug, Clone, Copy)]
pub enum ImportKind {
  /// A component import.
  Component,
  /// A types import.
  Types,
}

impl std::fmt::Display for ImportKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ImportKind::Component => write!(f, "component"),
      ImportKind::Types => write!(f, "types"),
    }
  }
}

impl Renderable for ImportDefinition {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    match self {
      ImportDefinition::Component(v) => v.render_config(source, root_config, env),
      ImportDefinition::Types(_) => Ok(()),
    }
  }
}
