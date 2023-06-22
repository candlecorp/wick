#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
pub(super) mod resources;
pub(super) mod triggers;

use asset_container::{AssetManager, Assets};
use tracing::trace;
use wick_asset_reference::{AssetReference, FetchOptions};
use wick_interface_types::TypeDefinition;
use wick_packet::RuntimeConfig;

pub use self::resources::*;
pub use self::triggers::*;
use super::common::component_definition::ComponentDefinition;
use super::common::package_definition::PackageConfig;
use super::{make_resolver, resolve, ImportBinding, ImportDefinition};
use crate::error::{ManifestError, ReferenceError};
use crate::import_cache::{setup_cache, ImportCache};
use crate::utils::RwOption;
use crate::{config, v1, Resolver, Result};

#[derive(Debug, Clone, Default, Builder, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(AssetReference))]
#[builder(setter(into))]
#[must_use]
/// A Wick application configuration.
///
/// An application configuration defines a wick application, its trigger, imported component, etc and can be executed
/// via `wick run`.
pub struct AppConfiguration {
  #[asset(skip)]
  /// The name of the application.
  pub(crate) name: String,

  #[asset(skip)]
  #[builder(setter(strip_option), default)]
  #[property(skip)]
  /// The source (i.e. url or file on disk) of the configuration.
  pub(crate) source: Option<PathBuf>,

  #[builder(setter(strip_option), default)]
  /// The metadata for the application.
  pub(crate) metadata: Option<config::Metadata>,

  #[builder(setter(strip_option), default)]
  /// The package configuration for this application.
  pub(crate) package: Option<PackageConfig>,

  #[builder(default)]
  /// The components that make up the application.
  pub(crate) import: HashMap<String, ImportBinding>,

  #[builder(default)]
  /// Any resources this application defines.
  pub(crate) resources: HashMap<String, ResourceBinding>,

  #[builder(default)]
  /// The triggers that initialize upon a `run` and make up the application.
  pub(crate) triggers: Vec<TriggerDefinition>,

  #[asset(skip)]
  #[doc(hidden)]
  #[builder(default)]
  pub(crate) root_config: Option<RuntimeConfig>,

  #[asset(skip)]
  #[builder(setter(skip))]
  #[property(skip)]
  #[doc(hidden)]
  pub(crate) type_cache: ImportCache,

  #[asset(skip)]
  pub(crate) options: Option<FetchOptions>,

  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  #[doc(hidden)]
  pub(crate) cached_types: RwOption<Vec<TypeDefinition>>,
}

impl AppConfiguration {
  /// Fetch/cache anything critical to the first use of this configuration.
  pub(crate) async fn setup_cache(&self, options: FetchOptions) -> Result<()> {
    setup_cache(
      &self.type_cache,
      self.import.values(),
      &self.cached_types,
      vec![],
      options,
    )
    .await
  }

  /// Get the package files
  pub fn package_files(&self) -> Assets<AssetReference> {
    self.package.assets()
  }

  /// Get the configuration item a binding points to.
  #[must_use]
  pub fn resolve_binding(&self, name: &str) -> Option<Result<OwnedConfigurationItem>> {
    let env = std::env::vars().collect();
    resolve(name, &self.import, &self.resources, self.root_config(), Some(&env))
  }

  /// Returns a function that resolves a binding to a configuration item.
  #[must_use]
  pub fn resolver(&self) -> Box<Resolver> {
    make_resolver(
      self.import.clone(),
      self.resources.clone(),
      self.root_config().cloned(),
      Some(std::env::vars().collect()),
    )
  }

  /// Return the underlying version of the source manifest.
  #[must_use]
  pub fn source(&self) -> Option<&Path> {
    self.source.as_deref()
  }

  /// Set the source location of the configuration.
  pub fn set_source(&mut self, source: &Path) {
    let mut source = source.to_path_buf();
    self.source = Some(source.clone());
    // Source is (should be) a file, so pop the filename before setting the baseurl.
    if !source.is_dir() {
      source.pop();
    }
    self.set_baseurl(&source);
  }

  /// Return the version of the application.
  #[must_use]
  pub fn version(&self) -> Option<&str> {
    self.metadata.as_ref().map(|m| m.version.as_str())
  }

  #[must_use]
  /// Get the application's imports.
  pub fn imports(&self) -> &HashMap<String, ImportBinding> {
    &self.import
  }

  /// Add a resource to the application configuration.
  pub fn add_resource(&mut self, name: impl AsRef<str>, resource: ResourceDefinition) {
    self
      .resources
      .insert(name.as_ref().to_owned(), ResourceBinding::new(name.as_ref(), resource));
  }

  /// Add a component to the application configuration.
  pub fn add_import(&mut self, name: impl AsRef<str>, import: ImportDefinition) {
    self
      .import
      .insert(name.as_ref().to_owned(), ImportBinding::new(name.as_ref(), import));
  }

  /// Generate V1 configuration yaml from this configuration.
  pub fn into_v1_yaml(self) -> Result<String> {
    let v1_manifest: v1::AppConfiguration = self.try_into()?;
    Ok(serde_yaml::to_string(&v1_manifest).unwrap())
  }

  /// Initialize the configuration with the given environment variables.
  pub fn initialize(&mut self, env: Option<&HashMap<String, String>>) -> Result<()> {
    // This pre-renders the application config's resources with access to the environment
    // so they're resulting value is intuitively based on where it was initially defined.
    let config = self.root_config().cloned();
    trace!(
      num_resources = self.resources.len(),
      ?config,
      "initializing app resources"
    );
    for resource in self.resources.values_mut() {
      resource.kind.render(config.as_ref(), env)?;
    }
    Ok(())
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
  pub fn try_component(&self) -> std::result::Result<&'a ComponentDefinition, ReferenceError> {
    match self {
      Self::Component(c) => Ok(c),
      _ => Err(ReferenceError::Component),
    }
  }
  /// Get the resource definition or return an error.
  pub fn try_resource(&self) -> std::result::Result<&'a ResourceDefinition, ReferenceError> {
    match self {
      Self::Resource(c) => Ok(c),
      _ => Err(ReferenceError::Resource),
    }
  }
}

/// A configuration item
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub enum OwnedConfigurationItem {
  /// A component definition.
  Component(ComponentDefinition),
  /// A resource definition.
  Resource(ResourceDefinition),
}

impl OwnedConfigurationItem {
  /// Get the component definition or return an error.
  pub fn try_component(&self) -> Result<ComponentDefinition> {
    match self {
      Self::Component(c) => Ok(c.clone()),
      _ => Err(ManifestError::Reference(ReferenceError::Component)),
    }
  }
  /// Get the resource definition or return an error.
  pub fn try_resource(&self) -> Result<ResourceDefinition> {
    match self {
      Self::Resource(c) => Ok(c.clone()),
      _ => Err(ManifestError::Reference(ReferenceError::Resource)),
    }
  }
}
