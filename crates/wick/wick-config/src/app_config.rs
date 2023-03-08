use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
mod resources;
mod triggers;
use tracing::debug;

pub use self::resources::*;
pub use self::triggers::*;
use crate::error::ManifestError;
use crate::{from_yaml, v1, ComponentKind, Error, Result};

#[derive(Debug, Clone, Default)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct AppConfiguration {
  source: Option<String>,
  version: u8,
  name: String,
  import: HashMap<String, ComponentKind>,
  resources: HashMap<String, ResourceDefinition>,
  triggers: Vec<TriggerDefinition>,
}

impl TryFrom<v1::AppConfiguration> for AppConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::AppConfiguration) -> Result<Self> {
    Ok(AppConfiguration {
      source: None,
      version: def.version,
      name: def.name,
      import: def.import.into_iter().map(|(k, v)| (k, v.into())).collect(),
      resources: def.resources.into_iter().map(|(k, v)| (k, v.into())).collect(),
      triggers: def.triggers.into_iter().map(|v| v.into()).collect(),
    })
  }
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
    let raw_version = raw.get("version").ok_or(Error::NoVersion)?;
    let version = raw_version
      .as_i64()
      .unwrap_or_else(|| -> i64 { raw_version.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1) });
    let manifest = match version {
      0 => panic!("no v0 implemented for app configuration"),
      1 => Ok(from_yaml::<v1::AppConfiguration>(src)?.try_into()?),
      -1 => Err(Error::NoVersion),
      _ => Err(Error::VersionError(version.to_string())),
    };

    debug!("Manifest: {:?}", manifest);
    manifest
  }

  /// Return the underlying version of the source manifest.
  #[must_use]
  pub fn version(&self) -> u8 {
    self.version
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
  pub fn imports(&self) -> &HashMap<String, ComponentKind> {
    &self.import
  }

  #[must_use]
  /// Get the application's resources.
  pub fn resources(&self) -> &HashMap<String, ResourceDefinition> {
    &self.resources
  }

  #[must_use]
  /// Get the application's triggers.
  pub fn triggers(&self) -> &Vec<TriggerDefinition> {
    &self.triggers
  }
}
