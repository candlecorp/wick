#![allow(missing_docs)] // delete when we move away from the `property` crate.

/// Specific component-level configuration and types.
pub mod components;

pub(crate) mod app_config;
pub(crate) mod common;
pub(crate) mod component_config;
pub(crate) mod configuration_tree;
pub(crate) mod import_cache;
pub(crate) mod lockdown_config;
pub(crate) mod permissions;
pub(crate) mod test_config;
pub(crate) mod types_config;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use app_config::*;
use asset_container::Asset;
pub use common::*;
pub use component_config::*;
pub use configuration_tree::*;
pub use lockdown_config::*;
pub use permissions::{Permissions, PermissionsBuilder};
pub use test_config::*;
use tracing::debug;
pub use types_config::*;
use wick_asset_reference::{AssetReference, FetchOptions};
use wick_interface_types::Field;
use wick_packet::validation::expect_configuration_matches;
use wick_packet::{Entity, RuntimeConfig};

use crate::load::resolve_configuration;
use crate::lockdown::Lockdown;
use crate::Error;

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
  /// The lockdown configuration that will validate a configuration is safe to run.
  pub(crate) lockdown_config: Option<LockdownConfiguration>,
}

impl UninitializedConfiguration {
  #[must_use]
  /// Create a new builder with the given manifest.
  pub fn new(manifest: WickConfiguration) -> Self {
    Self {
      manifest,
      root_config: None,
      env: None,
      lockdown_config: None,
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

impl Lockdown for UninitializedConfiguration {
  fn lockdown(&self, id: Option<&str>, lockdown: &LockdownConfiguration) -> Result<(), crate::lockdown::LockdownError> {
    self.manifest.lockdown(id, lockdown)?;
    Ok(())
  }
}

/// A catch-all enum for root-level Wick configurations.
#[derive(Debug, Clone, derive_asset_container::AssetManager, serde::Serialize)]
#[asset(asset(AssetReference))]
#[serde(untagged)]
pub enum WickConfiguration {
  /// A [component_config::ComponentConfiguration] configuration.
  Component(ComponentConfiguration),
  /// An [app_config::AppConfiguration] configuration.
  App(AppConfiguration),
  /// A [types_config::TypesConfiguration] configuration.
  Types(TypesConfiguration),
  /// A [test_config::TestConfiguration] configuration.
  Tests(TestConfiguration),
  /// A [lockdown_config::LockdownConfiguration] configuration.
  Lockdown(LockdownConfiguration),
}

impl WickConfiguration {
  /// Fetch a configuration and all referenced assets from a path.
  ///
  /// # Example
  ///
  /// ```rust
  /// # tokio_test::block_on(async {
  /// use wick_config::WickConfiguration;
  /// use wick_asset_reference::FetchOptions;
  ///
  /// let opts = FetchOptions::default();
  ///
  /// let manifest = WickConfiguration::fetch_all("path/to/manifest.yaml", opts).await?;
  /// # Ok::<_,anyhow::Error>(())
  /// # });
  /// ```
  pub async fn fetch_tree(
    path: impl Into<AssetReference> + Send,
    root_config: Option<RuntimeConfig>,
    root_env: HashMap<String, String>,
    options: FetchOptions,
  ) -> Result<ConfigurationTreeNode, Error> {
    let mut config = Self::fetch(path, options.clone()).await?;
    match config.manifest.kind() {
      ConfigurationKind::App | ConfigurationKind::Tests | ConfigurationKind::Lockdown => {
        config.set_env(root_env);
      }
      _ => {}
    }
    config.set_root_config(root_config);
    let config = config.finish()?;
    let mut node = ConfigurationTreeNode::new(Entity::LOCAL.into(), config);
    node.fetch_children(options).await?;

    Ok(node)
  }

  /// Fetch a configuration from a path.
  ///
  /// # Example
  ///
  /// ```rust
  /// # tokio_test::block_on(async {
  /// use wick_config::WickConfiguration;
  /// use wick_asset_reference::FetchOptions;
  ///
  /// let opts = FetchOptions::default();
  ///
  /// let manifest = WickConfiguration::fetch("path/to/manifest.yaml", opts).await?;
  /// # Ok::<_,anyhow::Error>(())
  /// # });
  /// ```
  ///
  pub async fn fetch(
    asset: impl Into<AssetReference> + Send,
    options: FetchOptions,
  ) -> Result<UninitializedConfiguration, Error> {
    let asset: AssetReference = asset.into();

    let bytes = asset.fetch(options.clone()).await?;
    let source = asset.path().unwrap_or_else(|e| PathBuf::from(format!("<ERROR:{}>", e)));
    let config = WickConfiguration::load_from_bytes(&bytes, &Some(source))?;
    config.manifest.update_baseurls();
    match &config.manifest {
      WickConfiguration::Component(c) => {
        c.setup_cache(options).await?;
      }
      WickConfiguration::App(c) => {
        c.setup_cache(options).await?;
      }
      WickConfiguration::Types(_) => {}
      WickConfiguration::Tests(_) => {}
      WickConfiguration::Lockdown(_) => {}
    }
    Ok(config)
  }

  /// Load a configuration from raw bytes. Pass in an optional source to track where the bytes came from.
  ///
  /// # Example
  ///
  /// ```rust
  /// # tokio_test::block_on(async {
  /// use wick_config::WickConfiguration;
  /// use std::path::PathBuf;
  ///
  /// let path = PathBuf::from("path/to/manifest.yaml");
  ///
  /// let bytes = std::fs::read(&path)?;
  ///
  /// let manifest = WickConfiguration::load_from_bytes(&bytes, &Some(path))?;
  /// # Ok::<_,anyhow::Error>(())
  /// # });
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
  /// # tokio_test::block_on(async {
  /// use std::path::PathBuf;
  /// use wick_config::WickConfiguration;
  ///
  /// let path = PathBuf::from("path/to/manifest.yaml");
  ///
  /// let string = std::fs::read_to_string(&path)?;
  ///
  /// let manifest = WickConfiguration::from_yaml(&string, &Some(path))?;
  /// # Ok::<_,anyhow::Error>(())
  /// # });
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
  /// # tokio_test::block_on(async {
  /// use wick_config::WickConfiguration;
  /// use wick_asset_reference::FetchOptions;
  ///
  /// let opts = FetchOptions::default();
  ///
  /// let manifest = WickConfiguration::fetch_all("path/to/manifest.yaml", opts).await?;
  /// let manifest = manifest.finish()?;
  ///
  /// let v1_yaml = manifest.into_v1_yaml()?;
  /// # Ok::<_,anyhow::Error>(())
  /// # });
  /// ```
  #[cfg(feature = "v1")]
  pub fn into_v1_yaml(self) -> Result<String, Error> {
    Ok(serde_yaml::to_string(&self.into_v1()?).unwrap())
  }

  /// Convert a WickConfiguration into a V1 configuration JSON value.
  ///
  /// # Example
  ///
  /// ```rust
  /// # tokio_test::block_on(async {
  /// use wick_config::WickConfiguration;
  /// use wick_asset_reference::FetchOptions;
  ///
  /// let opts = FetchOptions::default();
  ///
  /// let manifest = WickConfiguration::fetch_all("path/to/manifest.yaml", opts).await?;
  /// let manifest = manifest.finish()?;
  ///
  /// let v1_json = manifest.into_v1_json()?;
  /// # Ok::<_,anyhow::Error>(())
  /// # });
  /// ```
  #[cfg(feature = "v1")]
  pub fn into_v1_json(self) -> Result<serde_json::Value, Error> {
    Ok(serde_json::to_value(&self.into_v1()?).unwrap())
  }

  #[cfg(feature = "v1")]
  fn into_v1(self) -> Result<crate::v1::WickConfig, Error> {
    match self {
      WickConfiguration::Component(c) => Ok(crate::v1::WickConfig::ComponentConfiguration(c.try_into()?)),
      WickConfiguration::App(c) => Ok(crate::v1::WickConfig::AppConfiguration(c.try_into()?)),
      WickConfiguration::Types(c) => Ok(crate::v1::WickConfig::TypesConfiguration(c.try_into()?)),
      WickConfiguration::Tests(c) => Ok(crate::v1::WickConfig::TestConfiguration(c.try_into()?)),
      WickConfiguration::Lockdown(c) => Ok(crate::v1::WickConfig::LockdownConfiguration(c.try_into()?)),
    }
  }

  /// Get the name (if any) associated with the inner configuration.
  #[must_use]
  pub fn name(&self) -> Option<&str> {
    match self {
      WickConfiguration::Component(v) => v.name().map(|s| s.as_str()),
      WickConfiguration::App(v) => Some(v.name()),
      WickConfiguration::Types(v) => v.name().map(|s| s.as_str()),
      WickConfiguration::Tests(v) => v.name().map(|s| s.as_str()),
      WickConfiguration::Lockdown(_) => None,
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
      WickConfiguration::Lockdown(_v) => None,
    }
  }

  /// Validate this configuration is good.
  pub fn validate(&self) -> Result<(), Error> {
    match self {
      WickConfiguration::Component(v) => v.validate(),
      WickConfiguration::App(v) => v.validate(),
      WickConfiguration::Types(v) => v.validate(),
      WickConfiguration::Tests(v) => v.validate(),
      WickConfiguration::Lockdown(v) => v.validate(),
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
      WickConfiguration::Lockdown(_v) => Default::default(),
    }
  }

  /// Set the root runtime config for a [WickConfiguration].
  fn set_root_config(&mut self, env: Option<RuntimeConfig>) -> &mut Self {
    match self {
      WickConfiguration::App(v) => {
        v.root_config = env;
      }
      WickConfiguration::Component(v) => {
        v.root_config = env;
      }
      WickConfiguration::Types(_) => (),
      WickConfiguration::Tests(_) => (),
      WickConfiguration::Lockdown(_) => (),
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
      WickConfiguration::Lockdown(v) => v.env = env,
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
      WickConfiguration::Lockdown(_) => ConfigurationKind::Lockdown,
    }
  }

  /// Get the imports for the configuration, if any
  pub fn imports(&self) -> &[ImportBinding] {
    match self {
      WickConfiguration::Component(c) => c.import(),
      WickConfiguration::App(c) => c.import(),
      WickConfiguration::Types(_) => &[],
      WickConfiguration::Tests(_) => &[],
      WickConfiguration::Lockdown(_) => &[],
    }
  }

  /// Get the resources for the configuration, if any
  pub fn resources(&self) -> &[ResourceBinding] {
    match self {
      WickConfiguration::Component(c) => c.resources(),
      WickConfiguration::App(c) => c.resources(),
      WickConfiguration::Types(_) => &[],
      WickConfiguration::Tests(_) => &[],
      WickConfiguration::Lockdown(_) => &[],
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
      WickConfiguration::Lockdown(_) => None,
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
      WickConfiguration::Lockdown(_) => None,
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

  /// Unwrap the inner [LockdownConfiguration], returning an error if it is anything else.
  pub fn try_lockdown_config(self) -> Result<LockdownConfiguration, Error> {
    match self {
      WickConfiguration::Lockdown(v) => Ok(v),
      _ => Err(Error::UnexpectedConfigurationKind(
        ConfigurationKind::Lockdown,
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
      WickConfiguration::Tests(v) => {
        v.initialize()?;
      }
      WickConfiguration::Lockdown(v) => {
        v.initialize()?;
      }
    }
    self.update_baseurls();
    Ok(self)
  }

  /// Set the source of the configuration if it is not already set on load.
  pub fn set_source(&mut self, src: &Path) {
    match self {
      WickConfiguration::Component(v) => v.set_source(src),
      WickConfiguration::App(v) => v.set_source(src),
      WickConfiguration::Types(v) => v.set_source(src),
      WickConfiguration::Tests(v) => v.set_source(src),
      WickConfiguration::Lockdown(v) => v.set_source(src),
    }
  }

  fn update_baseurls(&self) {
    match self {
      WickConfiguration::Component(v) => v.update_baseurls(),
      WickConfiguration::App(v) => v.update_baseurls(),
      WickConfiguration::Types(v) => v.update_baseurls(),
      WickConfiguration::Tests(v) => v.update_baseurls(),
      WickConfiguration::Lockdown(v) => v.update_baseurls(),
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
      WickConfiguration::Lockdown(v) => v.source.as_deref(),
    }
  }
}

impl Lockdown for WickConfiguration {
  fn lockdown(&self, id: Option<&str>, lockdown: &LockdownConfiguration) -> Result<(), crate::lockdown::LockdownError> {
    match self {
      WickConfiguration::Component(v) => v.lockdown(id, lockdown),
      WickConfiguration::App(v) => v.lockdown(id, lockdown),
      WickConfiguration::Types(_) => Ok(()),
      WickConfiguration::Tests(_) => Ok(()),
      WickConfiguration::Lockdown(_) => Ok(()),
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
  /// A [lockdown_config::LockdownConfiguration] configuration.
  Lockdown,
}

impl std::fmt::Display for ConfigurationKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConfigurationKind::App => write!(f, "wick/app"),
      ConfigurationKind::Component => write!(f, "wick/component"),
      ConfigurationKind::Types => write!(f, "wick/types"),
      ConfigurationKind::Tests => write!(f, "wick/tests"),
      ConfigurationKind::Lockdown => write!(f, "wick/lockdown"),
    }
  }
}
