use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
mod resources;
mod triggers;
use tracing::debug;

pub use self::resources::*;
pub use self::triggers::*;
use crate::error::ReferenceError;
use crate::{from_yaml, v1, BoundComponent, ComponentDefinition, Error, Result};

#[derive(Debug, Clone, Default)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct AppConfiguration {
  pub name: String,
  pub(crate) source: Option<String>,
  pub(crate) format: u32,
  pub(crate) version: String,
  pub(crate) import: HashMap<String, BoundComponent>,
  pub(crate) resources: HashMap<String, BoundResource>,
  pub(crate) triggers: Vec<TriggerDefinition>,
}

impl AppConfiguration {
  /// Load struct from file by trying all the supported file formats.
  pub fn load_from_file(path: impl AsRef<Path>) -> Result<AppConfiguration> {
    let path = path.as_ref();
    if !path.exists() {
      return Err(Error::FileNotFound(path.to_string_lossy().into()));
    }
    debug!("Reading manifest from {}", path.to_string_lossy());
    let contents = read_to_string(path)?;
    let mut manifest = Self::from_yaml(&contents)?;
    manifest.source = Some(path.to_string_lossy().to_string());
    Ok(manifest)
  }

  /// Load struct from bytes by attempting to parse all the supported file formats.
  pub fn load_from_bytes(source: Option<String>, bytes: &[u8]) -> Result<AppConfiguration> {
    let contents = String::from_utf8_lossy(bytes);
    let mut manifest = Self::from_yaml(&contents)?;
    manifest.source = source;
    Ok(manifest)
  }

  /// Load as YAML.
  pub fn from_yaml(src: &str) -> Result<AppConfiguration> {
    debug!("Trying to parse manifest as yaml");
    let raw: serde_yaml::Value = from_yaml(src)?;
    debug!("Yaml parsed successfully");
    let raw_version = raw.get("format").ok_or(Error::NoFormat)?;
    let version = raw_version
      .as_i64()
      .unwrap_or_else(|| -> i64 { raw_version.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1) });
    let manifest = match version {
      0 => panic!("no v0 implemented for app configuration"),
      1 => Ok(from_yaml::<v1::AppConfiguration>(src)?.try_into()?),
      -1 => Err(Error::NoFormat),
      _ => Err(Error::VersionError(version.to_string())),
    };

    debug!("Manifest: {:?}", manifest);
    manifest
  }

  /// Get the configuration item a binding points to.
  #[must_use]
  pub fn resolve_binding(&self, name: &str) -> Option<ConfigurationItem> {
    if let Some(component) = self.import.get(name) {
      return Some(ConfigurationItem::Component(&component.kind));
    }
    None
  }

  /// Return the underlying version of the source manifest.
  #[must_use]
  pub fn version(&self) -> &str {
    &self.version
  }

  /// Return the underlying version of the application.
  #[must_use]
  pub fn format(&self) -> u32 {
    self.format
  }

  /// Return the underlying version of the source manifest.
  #[must_use]
  pub fn source(&self) -> &Option<String> {
    &self.source
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn name(&self) -> &str {
    &self.name
  }

  #[must_use]
  /// Get the application's imports.
  pub fn imports(&self) -> &HashMap<String, BoundComponent> {
    &self.import
  }

  #[must_use]
  /// Get the application's resources.
  pub fn resources(&self) -> &HashMap<String, BoundResource> {
    &self.resources
  }

  #[must_use]
  /// Get the application's triggers.
  pub fn triggers(&self) -> &Vec<TriggerDefinition> {
    &self.triggers
  }
}

/// A configuration item
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub enum ConfigurationItem<'a> {
  /// A component definition.
  Component(&'a ComponentDefinition),
  /// A resource definition.
  Resource(&'a ResourceDefinition),
}

impl<'a> ConfigurationItem<'a> {
  /// Get the component definition or return an error.
  pub fn component(&self) -> std::result::Result<&'a ComponentDefinition, ReferenceError> {
    match self {
      ConfigurationItem::Component(c) => Ok(c),
      _ => Err(ReferenceError::Component),
    }
  }
}
