pub(crate) mod app_config;
pub(crate) mod common;
pub(crate) mod component_config;
/// Specific component-level configuration and types.
pub mod components;
pub(crate) mod test_config;
pub(crate) mod types_config;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use app_config::*;
use asset_container::{Asset, AssetFlags, AssetManager};
pub use common::*;
pub use component_config::*;
pub use test_config::*;
use tokio::fs::read_to_string;
use tracing::debug;
pub use types_config::*;
pub use wick_asset_reference::{AssetReference, FetchOptions};
use wick_packet::RuntimeConfig;

use crate::error::ManifestError;
use crate::utils::from_yaml;
use crate::{v0, v1, Error, Resolver};

#[derive(Debug, Clone, Copy)]
/// The kind of configuration loaded.
#[must_use]
pub enum ConfigurationKind {
  /// An [app_config::AppConfiguration] configuration.
  App,
  /// A [component_config::ComponentConfiguration] configuration.
  Component,
  /// A [types_config::TypesConfiguration] configuration.
  Types,
  /// A [test_config::TestConfiguration] configuration.
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
    if asset.get_asset_flags() == AssetFlags::Lazy {
      continue;
    }
    asset.fetch(options.clone()).await?;
  }
  Ok(())
}

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
/// A catch-all enum for root-level Wick configurations.
pub enum WickConfiguration {
  /// A [component_config::ComponentConfiguration] configuration.
  Component(ComponentConfiguration),
  /// An [app_config::AppConfiguration] configuration.
  App(AppConfiguration),
  /// A [types_config::TypesConfiguration] configuration.
  Types(TypesConfiguration),
  /// A [test_config::TestConfiguration] configuration.
  Tests(TestConfiguration),
}

impl WickConfiguration {
  /// Fetch a configuration and all referenced assets from a path.
  ///
  /// # Example
  ///
  /// ```rust
  /// use wick_config::WickConfiguration;
  /// use wick_asset_reference::FetchOptions;
  ///
  /// let opts = FetchOptions::default();
  ///
  /// let manifest = WickConfiguration::fetch_all("path/to/manifest.yaml", opts).await?;
  /// ```
  pub async fn fetch_all(path: impl Into<String> + Send, options: FetchOptions) -> Result<Self, Error> {
    let config = Self::fetch(path, options.clone()).await?;
    config.fetch_assets(options).await?;
    Ok(config)
  }

  /// Fetch a configuration from a path.
  ///
  /// # Example
  ///
  /// ```rust
  ///
  /// use wick_config::WickConfiguration;
  /// use wick_asset_reference::FetchOptions;
  ///
  /// let opts = FetchOptions::default();
  ///
  /// let manifest = WickConfiguration::fetch("path/to/manifest.yaml", opts).await?;
  /// ```
  ///
  pub async fn fetch(path: impl Into<String> + Send, options: FetchOptions) -> Result<Self, Error> {
    let path = path.into();
    let location = AssetReference::new(&path);

    let bytes = location.fetch(options.clone()).await?;
    let source = location
      .path()
      .unwrap_or_else(|e| PathBuf::from(format!("<ERROR:{}>", e)));
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

  /// Load a configuration from raw bytes. Pass in an optional source to track where the bytes came from.
  ///
  /// # Example
  ///
  /// ```rust
  ///
  /// use wick_config::WickConfiguration;
  ///
  /// let path = PathBuf::from("path/to/manifest.yaml");
  ///
  /// let bytes = std::fs::read(&path)?;
  ///
  /// let manifest = WickConfiguration::load_from_bytes(bytes, &Some(path))?;
  ///
  /// ```
  pub fn load_from_bytes(bytes: &[u8], source: &Option<PathBuf>) -> Result<Self, Error> {
    let string = &String::from_utf8(bytes.to_vec()).map_err(|_| Error::Utf8)?;

    resolve_configuration(string, source)
  }

  /// Load a configuration from a string. Pass in an optional source to track where the string came from.
  ///
  /// # Example
  ///
  /// ```rust
  ///
  /// use wick_config::WickConfiguration;
  ///
  /// let path = PathBuf::from("path/to/manifest.yaml");
  ///
  /// let string = std::fs::read_to_string(&path)?;
  ///
  /// let manifest = WickConfiguration::load_from_string(string, &Some(path))?;
  ///
  /// ```
  ///
  pub fn from_yaml(src: &str, source: &Option<PathBuf>) -> Result<Self, Error> {
    resolve_configuration(src, source)
  }

  /// Convert a WickConfiguration into V1 configuration yaml source.
  ///
  /// # Example
  ///
  /// ```rust
  ///
  /// use wick_config::WickConfiguration;
  /// use wick_asset_reference::FetchOptions;
  ///
  /// let opts = FetchOptions::default();
  ///
  /// let manifest = WickConfiguration::fetch_all("path/to/manifest.yaml", opts).await?;
  ///
  /// let v1_yaml = manifest.into_v1_yaml()?;
  ///
  /// ```
  pub fn into_v1_yaml(self) -> Result<String, Error> {
    let v1_manifest = match self {
      WickConfiguration::Component(c) => v1::WickConfig::ComponentConfiguration(c.try_into()?),
      WickConfiguration::App(c) => v1::WickConfig::AppConfiguration(c.try_into()?),
      WickConfiguration::Types(c) => v1::WickConfig::TypesConfiguration(c.try_into()?),
      WickConfiguration::Tests(c) => v1::WickConfig::TestConfiguration(c.try_into()?),
    };

    Ok(serde_yaml::to_string(&v1_manifest).unwrap())
  }

  /// Get the name (if any) associated with the inner configuration.
  #[must_use]
  pub fn name(&self) -> Option<&str> {
    match self {
      WickConfiguration::Component(v) => v.name().map(|s| s.as_str()),
      WickConfiguration::App(v) => Some(v.name()),
      WickConfiguration::Types(v) => v.name().map(|s| s.as_str()),
      WickConfiguration::Tests(v) => v.name().map(|s| s.as_str()),
    }
  }

  /// Get the metadata (if any) associated with the inner configuration.
  #[must_use]
  pub fn metadata(&self) -> Option<&Metadata> {
    match self {
      WickConfiguration::Component(v) => v.metadata(),
      WickConfiguration::App(v) => v.metadata(),
      WickConfiguration::Types(v) => v.metadata(),
      WickConfiguration::Tests(_v) => None,
    }
  }

  /// Get the runtime configuration (if any) associated with the inner configuration.
  #[must_use]
  pub fn root_config(&self) -> Option<&RuntimeConfig> {
    match self {
      WickConfiguration::Component(v) => v.root_config(),
      WickConfiguration::App(v) => v.root_config(),
      WickConfiguration::Types(_v) => None,
      WickConfiguration::Tests(_v) => None,
    }
  }

  /// Get the kind of the inner configuration.
  pub fn kind(&self) -> ConfigurationKind {
    match self {
      WickConfiguration::Component(_) => ConfigurationKind::Component,
      WickConfiguration::App(_) => ConfigurationKind::App,
      WickConfiguration::Types(_) => ConfigurationKind::Types,
      WickConfiguration::Tests(_) => ConfigurationKind::Tests,
    }
  }

  /// Get the version (if any) associated with the inner configuration.
  #[must_use]
  pub fn version(&self) -> Option<&str> {
    match self {
      WickConfiguration::Component(v) => v.version(),
      WickConfiguration::App(v) => v.version(),
      WickConfiguration::Types(v) => v.version(),
      WickConfiguration::Tests(_) => None,
    }
  }

  /// Get the package configuration (if any) associated with the inner configuration.
  #[must_use]
  pub fn package(&self) -> Option<&PackageConfig> {
    match self {
      WickConfiguration::Component(v) => v.package(),
      WickConfiguration::App(v) => v.package(),
      WickConfiguration::Types(v) => v.package(),
      WickConfiguration::Tests(_) => None,
    }
  }

  /// Unwrap the inner [ComponentConfiguration], returning an error if it is anything else.
  pub fn try_component_config(self) -> Result<ComponentConfiguration, Error> {
    match self {
      WickConfiguration::Component(v) => Ok(v),
      _ => Err(Error::UnexpectedConfigurationKind(
        ConfigurationKind::Component,
        self.kind(),
      )),
    }
  }

  /// Unwrap the inner [AppConfiguration], returning an error if it is anything else.
  pub fn try_app_config(self) -> Result<AppConfiguration, Error> {
    match self {
      WickConfiguration::App(v) => Ok(v),
      _ => Err(Error::UnexpectedConfigurationKind(ConfigurationKind::App, self.kind())),
    }
  }

  /// Unwrap the inner [TestConfiguration], returning an error if it is anything else.
  pub fn try_test_config(self) -> Result<TestConfiguration, Error> {
    match self {
      WickConfiguration::Tests(v) => Ok(v),
      _ => Err(Error::UnexpectedConfigurationKind(
        ConfigurationKind::Tests,
        self.kind(),
      )),
    }
  }

  /// Unwrap the inner [TypesConfiguration], returning an error if it is anything else.
  pub fn try_types_config(self) -> Result<TypesConfiguration, Error> {
    match self {
      WickConfiguration::Types(v) => Ok(v),
      _ => Err(Error::UnexpectedConfigurationKind(
        ConfigurationKind::Types,
        self.kind(),
      )),
    }
  }

  /// Initialize the configuration with the given environment variables.
  pub fn initialize(&mut self, env: Option<&HashMap<String, String>>) -> Result<(), Error> {
    match self {
      WickConfiguration::Component(v) => v.initialize(env),
      WickConfiguration::App(v) => v.initialize(env),
      WickConfiguration::Types(_) => Ok(()),
      WickConfiguration::Tests(_) => Ok(()),
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
    let contents = read_to_string(path)
      .await
      .map_err(|_| Error::LoadError(path.display().to_string()))?;
    let manifest = Self::from_yaml(&contents, &Some(path.to_path_buf()))?;
    Ok(manifest)
  }

  #[doc(hidden)]
  pub fn load_from_file_sync(path: impl AsRef<Path>) -> Result<Self, Error> {
    let path = path.as_ref();

    if !path.exists() {
      return Err(Error::FileNotFound(path.to_string_lossy().into()));
    }
    debug!("Reading manifest from {}", path.to_string_lossy());
    let contents = std::fs::read_to_string(path).map_err(|_| Error::LoadError(path.display().to_string()))?;
    let manifest = Self::from_yaml(&contents, &Some(path.to_path_buf()))?;
    Ok(manifest)
  }

  /// Set the source of the configuration if it is not already set on load.
  pub fn set_source(&mut self, src: &Path) {
    match self {
      WickConfiguration::Component(v) => v.set_source(src),
      WickConfiguration::App(v) => v.set_source(src),
      WickConfiguration::Types(v) => v.set_source(src),
      WickConfiguration::Tests(v) => v.set_source(src),
    }
  }

  /// Get the source of the configuration.
  #[must_use]
  pub fn source(&self) -> Option<&Path> {
    match self {
      WickConfiguration::Component(v) => v.source.as_deref(),
      WickConfiguration::App(v) => v.source.as_deref(),
      WickConfiguration::Types(v) => v.source.as_deref(),
      WickConfiguration::Tests(v) => v.source.as_deref(),
    }
  }
}

fn resolve_configuration(src: &str, source: &Option<PathBuf>) -> Result<WickConfiguration, Error> {
  let raw: serde_yaml::Value = from_yaml(src, source)?;

  let raw_version = raw.get("format");
  let raw_kind = raw.get("kind");
  let version = if raw_kind.is_some() {
    1
  } else {
    let raw_version = raw_version.ok_or(Error::NoFormat(source.clone()))?;
    raw_version
      .as_i64()
      .unwrap_or_else(|| -> i64 { raw_version.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1) })
  };
  // re-parse the yaml into the correct version from string again for location info.
  match version {
    0 => {
      let host_config = serde_yaml::from_str::<v0::HostManifest>(src)
        .map_err(|e| Error::YamlError(source.clone(), e.to_string(), e.location()))?;
      Ok(WickConfiguration::Component(host_config.try_into()?))
    }
    1 => {
      let base_config = serde_yaml::from_str::<v1::WickConfig>(src)
        .map_err(|e| Error::YamlError(source.clone(), e.to_string(), e.location()))?;
      let mut config: WickConfiguration = base_config.try_into()?;
      if let Some(src) = source {
        config.set_source(src);
      }
      Ok(config)
    }
    -1 => Err(Error::NoFormat(source.clone())),
    _ => Err(Error::VersionError(version.to_string())),
  }
}

pub(crate) fn make_resolver(
  imports: HashMap<String, ImportBinding>,
  resources: HashMap<String, ResourceBinding>,
  runtime_config: Option<RuntimeConfig>,
  env: Option<HashMap<String, String>>,
) -> Box<Resolver> {
  Box::new(move |name| resolve(name, &imports, &resources, runtime_config.as_ref(), env.as_ref()))
}

pub(crate) fn resolve(
  name: &str,
  imports: &HashMap<String, ImportBinding>,
  resources: &HashMap<String, ResourceBinding>,
  runtime_config: Option<&RuntimeConfig>,
  env: Option<&HashMap<String, String>>,
) -> Option<Result<OwnedConfigurationItem, ManifestError>> {
  if let Some(import) = imports.get(name) {
    if let ImportDefinition::Component(component) = &import.kind {
      let mut component = component.clone();
      return Some(match component.render(runtime_config, env) {
        Ok(_) => Ok(OwnedConfigurationItem::Component(component)),
        Err(e) => Err(e),
      });
    }
  }
  if let Some(resource) = resources.get(name) {
    let mut resource = resource.kind.clone();
    return Some(match resource.render(runtime_config, env) {
      Ok(_) => Ok(OwnedConfigurationItem::Resource(resource)),
      Err(e) => Err(e),
    });
  }
  None
}
