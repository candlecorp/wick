pub mod app_config;
pub mod common;
pub mod component_config;
pub mod components;
pub mod test_config;
pub mod types_config;

use std::collections::HashMap;
use std::path::Path;

pub use app_config::*;
use asset_container::{Asset, AssetManager};
pub use common::*;
pub use component_config::{FlowOperation, *};
pub use test_config::*;
use tokio::fs::read_to_string;
use tracing::debug;
pub use types_config::*;
pub use wick_asset_reference::{AssetReference, FetchOptions};

use crate::utils::{from_bytes, from_yaml};
use crate::{v0, v1, Error, Resolver};

#[derive(Debug, Clone, Copy)]
#[must_use]
pub enum ConfigurationKind {
  App,
  Component,
  Types,
  Tests,
}

impl std::fmt::Display for ConfigurationKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConfigurationKind::App => write!(f, "wick/app"),
      ConfigurationKind::Component => write!(f, "wick/component"),
      ConfigurationKind::Types => write!(f, "wick/types"),
      ConfigurationKind::Tests => write!(f, "wick/tests"),
    }
  }
}

async fn fetch_all(
  asset_manager: &(dyn AssetManager<Asset = AssetReference> + Send + Sync),
  options: FetchOptions,
) -> Result<(), Error> {
  for asset in asset_manager.assets().iter() {
    asset.fetch(options.clone()).await?;
  }
  Ok(())
}

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(AssetReference)]

pub enum WickConfiguration {
  Component(ComponentConfiguration),
  App(AppConfiguration),
  Types(TypesConfiguration),
  Tests(TestConfiguration),
}

impl WickConfiguration {
  pub async fn fetch_all(path: impl AsRef<str> + Send, options: FetchOptions) -> Result<Self, Error> {
    let config = Self::fetch(path, options.clone()).await?;
    config.fetch_assets(options).await?;
    Ok(config)
  }

  pub async fn fetch(path: impl AsRef<str> + Send, options: FetchOptions) -> Result<Self, Error> {
    let path = path.as_ref();
    let location = AssetReference::new(path);

    let bytes = location
      .fetch(options.clone())
      .await
      .map_err(|e| Error::LoadError(path.to_owned(), e.to_string()))?;
    let source = location.path().unwrap_or_else(|e| format!("<ERROR:{}>", e));
    let config = WickConfiguration::load_from_bytes(&bytes, &Some(source))?;
    match &config {
      WickConfiguration::Component(c) => {
        c.setup_cache(options).await?;
      }
      WickConfiguration::App(c) => {
        c.setup_cache(options).await?;
      }
      WickConfiguration::Types(_) => {}
      WickConfiguration::Tests(_) => {}
    }
    Ok(config)
  }

  async fn fetch_assets(&self, options: FetchOptions) -> Result<(), Error> {
    match self {
      WickConfiguration::Component(c) => fetch_all(c, options).await,
      WickConfiguration::App(c) => fetch_all(c, options).await,
      WickConfiguration::Types(c) => fetch_all(c, options).await,
      WickConfiguration::Tests(c) => fetch_all(c, options).await,
    }
  }

  pub fn load_from_bytes(bytes: &[u8], source: &Option<String>) -> Result<Self, Error> {
    debug!(source=?source,"Trying to parse manifest bytes as yaml");
    let raw: serde_yaml::Value = from_bytes(bytes, source)?;
    resolve_configuration(raw, source)
  }

  pub fn from_yaml(src: &str, source: &Option<String>) -> Result<Self, Error> {
    debug!(source=?source,"Trying to parse manifest as yaml");
    let raw: serde_yaml::Value = from_yaml(src, source)?;
    resolve_configuration(raw, source)
  }

  pub fn kind(&self) -> ConfigurationKind {
    match self {
      WickConfiguration::Component(_) => ConfigurationKind::Component,
      WickConfiguration::App(_) => ConfigurationKind::App,
      WickConfiguration::Types(_) => ConfigurationKind::Types,
      WickConfiguration::Tests(_) => ConfigurationKind::Tests,
    }
  }

  pub fn try_component_config(self) -> Result<ComponentConfiguration, Error> {
    match self {
      WickConfiguration::Component(v) => Ok(v),
      _ => Err(Error::UnexpectedConfigurationKind(
        ConfigurationKind::Component,
        self.kind(),
      )),
    }
  }

  pub fn try_app_config(self) -> Result<AppConfiguration, Error> {
    match self {
      WickConfiguration::App(v) => Ok(v),
      _ => Err(Error::UnexpectedConfigurationKind(ConfigurationKind::App, self.kind())),
    }
  }

  pub fn try_test_config(self) -> Result<TestConfiguration, Error> {
    match self {
      WickConfiguration::Tests(v) => Ok(v),
      _ => Err(Error::UnexpectedConfigurationKind(
        ConfigurationKind::Tests,
        self.kind(),
      )),
    }
  }

  pub fn try_types_config(self) -> Result<TypesConfiguration, Error> {
    match self {
      WickConfiguration::Types(v) => Ok(v),
      _ => Err(Error::UnexpectedConfigurationKind(
        ConfigurationKind::Types,
        self.kind(),
      )),
    }
  }

  /// Load struct from file by trying all the supported file formats.
  pub async fn load_from_file(path: impl AsRef<Path> + Send) -> Result<Self, Error> {
    let path = path.as_ref();
    let pathstr = path.to_string_lossy();
    if !path.exists() {
      return Err(Error::FileNotFound(pathstr.to_string()));
    }
    debug!("Reading manifest from {}", path.to_string_lossy());
    let contents = read_to_string(path).await.map_err(|e| {
      Error::LoadError(
        #[allow(clippy::expect_used)]
        path.display().to_string(),
        e.to_string(),
      )
    })?;
    let manifest = Self::from_yaml(&contents, &Some(pathstr.to_string()))?;
    Ok(manifest)
  }

  #[doc(hidden)]
  pub fn load_from_file_sync(path: impl AsRef<Path>) -> Result<Self, Error> {
    let path = path.as_ref();

    if !path.exists() {
      return Err(Error::FileNotFound(path.to_string_lossy().into()));
    }
    debug!("Reading manifest from {}", path.to_string_lossy());
    let contents = std::fs::read_to_string(path).map_err(|e| {
      Error::LoadError(
        #[allow(clippy::expect_used)]
        path.display().to_string(),
        e.to_string(),
      )
    })?;
    let manifest = Self::from_yaml(&contents, &Some(path.display().to_string()))?;
    Ok(manifest)
  }

  pub fn set_source(&mut self, src: String) {
    match self {
      WickConfiguration::Component(v) => v.set_source(src),
      WickConfiguration::App(v) => v.set_source(src),
      WickConfiguration::Types(v) => v.set_source(src),
      WickConfiguration::Tests(v) => v.set_source(src),
    }
  }

  #[must_use]
  pub fn source(&self) -> &Option<String> {
    match self {
      WickConfiguration::Component(v) => &v.source,
      WickConfiguration::App(v) => &v.source,
      WickConfiguration::Types(v) => &v.source,
      WickConfiguration::Tests(v) => &v.source,
    }
  }
}

fn resolve_configuration(raw: serde_yaml::Value, source: &Option<String>) -> Result<WickConfiguration, Error> {
  debug!("Yaml parsed successfully");
  let raw_version = raw.get("format");
  let raw_kind = raw.get("kind");
  let version = if raw_kind.is_some() {
    1
  } else {
    let raw_version = raw_version.ok_or(Error::NoFormat)?;
    raw_version
      .as_i64()
      .unwrap_or_else(|| -> i64 { raw_version.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1) })
  };

  let manifest = match version {
    0 => {
      let host_config = serde_yaml::from_value::<v0::HostManifest>(raw)
        .map_err(|e| Error::YamlError(source.as_ref().map(|v| v.to_string()), e.to_string()))?;
      Ok(WickConfiguration::Component(host_config.try_into()?))
    }
    1 => {
      let base_config = serde_yaml::from_value::<v1::WickConfig>(raw)
        .map_err(|e| Error::YamlError(source.as_ref().map(|v| v.to_string()), e.to_string()))?;
      let mut config: WickConfiguration = base_config.try_into()?;
      if let Some(src) = source {
        config.set_source(src.clone());
      }
      Ok(config)
    }
    -1 => Err(Error::NoFormat),
    _ => Err(Error::VersionError(version.to_string())),
  };

  debug!("Manifest: {:?}", manifest);
  manifest
}

pub(crate) fn make_resolver(
  imports: HashMap<String, ImportBinding>,
  resources: HashMap<String, ResourceBinding>,
) -> Box<Resolver> {
  Box::new(move |name| {
    if let Some(import) = imports.get(name) {
      if let ImportDefinition::Component(component) = &import.kind {
        return Some(OwnedConfigurationItem::Component(component.clone()));
      }
    }
    if let Some(resource) = resources.get(name) {
      return Some(OwnedConfigurationItem::Resource(resource.kind.clone()));
    }
    None
  })
}
