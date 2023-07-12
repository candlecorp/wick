#![allow(missing_docs)] // delete when we move away from the `property` crate.

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
use asset_container::Asset;
pub use common::*;
pub use component_config::*;
pub use test_config::*;
use tokio::fs::read_to_string;
use tracing::debug;
pub use types_config::*;
use wick_asset_reference::{AssetReference, FetchOptions};
use wick_interface_types::Field;
use wick_packet::validation::expect_configuration_matches;
use wick_packet::RuntimeConfig;

use crate::utils::{fetch_all, resolve_configuration};
use crate::{v1, Error};

#[derive(Debug, Clone, property::Property)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
/// A builder for [WickConfiguration].
pub struct UninitializedConfiguration {
  /// The manifest to use as a base.
  pub(crate) manifest: WickConfiguration,
  /// The root configuration to use when rendering internal configuration templates.
  pub(crate) root_config: Option<RuntimeConfig>,
  /// The environment this configuration can use when rendering internal configuration templates.
  pub(crate) env: Option<HashMap<String, String>>,
}

impl UninitializedConfiguration {
  #[must_use]
  /// Create a new builder with the given manifest.
  pub fn new(manifest: WickConfiguration) -> Self {
    Self {
      manifest,
      root_config: None,
      env: None,
    }
  }

  /// Return the inner, uninitialized [WickConfiguration].
  #[must_use]
  pub fn into_inner(self) -> WickConfiguration {
    self.manifest
  }

  /// Build, initialize and return a [WickConfiguration].
  pub fn finish(mut self) -> Result<WickConfiguration, Error> {
    debug!(root_config=?self.root_config, env=?self.env.as_ref().map(|c|format!("{} variables",c.len())), "initializing configuration");

    expect_configuration_matches(
      self
        .manifest
        .source()
        .map_or("<unknown>", |p| p.to_str().unwrap_or("<invalid>")),
      self.root_config.as_ref(),
      self.manifest.config(),
    )
    .map_err(Error::ConfigurationInvalid)?;
    self.manifest.set_env(self.env);
    self.manifest.set_root_config(self.root_config);
    self.manifest.initialize()?;
    self.manifest.validate()?;
    Ok(self.manifest)
  }
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
  pub async fn fetch_all(
    path: impl Into<String> + Send,
    options: FetchOptions,
  ) -> Result<UninitializedConfiguration, Error> {
    let config = Self::fetch(path, options.clone()).await?;
    config.manifest.fetch_assets(options).await?;
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
  pub async fn fetch(
    path: impl Into<String> + Send,
    options: FetchOptions,
  ) -> Result<UninitializedConfiguration, Error> {
    let path = path.into();
    let location = AssetReference::new(&path);

    let bytes = location.fetch(options.clone()).await?;
    let source = location
      .path()
      .unwrap_or_else(|e| PathBuf::from(format!("<ERROR:{}>", e)));
    let config = WickConfiguration::load_from_bytes(&bytes, &Some(source))?;
    match &config.manifest {
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
  pub fn load_from_bytes(bytes: &[u8], source: &Option<PathBuf>) -> Result<UninitializedConfiguration, Error> {
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
  pub fn from_yaml(src: &str, source: &Option<PathBuf>) -> Result<UninitializedConfiguration, Error> {
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

  /// Validate this configuration is good.
  pub fn validate(&self) -> Result<(), Error> {
    match self {
      WickConfiguration::Component(v) => v.validate(),
      WickConfiguration::App(v) => v.validate(),
      WickConfiguration::Types(v) => v.validate(),
      WickConfiguration::Tests(v) => v.validate(),
    }
  }

  /// Get the runtime configuration (if any) associated with the inner configuration.
  #[must_use]
  fn config(&self) -> &[Field] {
    match self {
      WickConfiguration::Component(v) => v.config(),
      WickConfiguration::App(_v) => Default::default(),
      WickConfiguration::Types(_v) => Default::default(),
      WickConfiguration::Tests(_v) => Default::default(),
    }
  }

  /// Set the root runtime config for a [WickConfiguration].
  fn set_root_config(&mut self, env: Option<RuntimeConfig>) -> &mut Self {
    match self {
      WickConfiguration::App(ref mut v) => {
        v.root_config = env;
      }
      WickConfiguration::Component(v) => {
        v.root_config = env;
      }
      WickConfiguration::Types(_) => (),
      WickConfiguration::Tests(_) => (),
    }
    self
  }

  /// Set the environment variables for a [WickConfiguration].
  fn set_env(&mut self, env: Option<HashMap<String, String>>) -> &mut Self {
    match self {
      WickConfiguration::App(ref mut v) => {
        v.env = env;
      }
      WickConfiguration::Component(_) => (),
      WickConfiguration::Types(_) => (),
      WickConfiguration::Tests(_) => (),
    }
    self
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

  /// Initialize the configuration.
  fn initialize(&mut self) -> Result<&Self, Error> {
    match self {
      WickConfiguration::Component(v) => {
        v.initialize()?;
      }
      WickConfiguration::App(v) => {
        v.initialize()?;
      }
      WickConfiguration::Types(_) => (),
      WickConfiguration::Tests(_) => (),
    }
    Ok(self)
  }

  /// Load struct from file by trying all the supported file formats.
  pub async fn load_from_file(path: impl AsRef<Path> + Send) -> Result<UninitializedConfiguration, Error> {
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
  pub fn load_from_file_sync(path: impl AsRef<Path>) -> Result<UninitializedConfiguration, Error> {
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
