use std::collections::HashMap;
use std::path::{Path, PathBuf};
pub(super) mod resources;
pub(super) mod triggers;

use asset_container::{AssetManager, Assets};
use wick_asset_reference::{AssetReference, FetchOptions};
use wick_interface_types::TypeDefinition;

pub use self::resources::*;
pub use self::triggers::*;
use super::common::component_definition::ComponentDefinition;
use super::common::package_definition::PackageConfig;
use super::{make_resolver, ImportBinding, ImportDefinition};
use crate::error::ReferenceError;
use crate::import_cache::{setup_cache, ImportCache};
use crate::utils::RwOption;
use crate::{config, v1, Resolver, Result};

#[derive(Debug, Clone, Default, Builder, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(disable), mut(disable))]
#[asset(asset(AssetReference))]
#[builder(setter(into))]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct AppConfiguration {
  #[asset(skip)]
  pub(crate) name: String,
  #[asset(skip)]
  #[builder(setter(strip_option), default)]
  #[property(skip)]
  pub(crate) source: Option<PathBuf>,
  #[builder(setter(strip_option), default)]
  #[property(skip)]
  pub(crate) metadata: Option<config::Metadata>,
  #[builder(default)]
  pub(crate) import: HashMap<String, ImportBinding>,
  #[builder(default)]
  pub(crate) resources: HashMap<String, ResourceBinding>,
  #[builder(default)]
  pub(crate) triggers: Vec<TriggerDefinition>,
  #[asset(skip)]
  #[builder(setter(skip))]
  #[property(skip)]
  pub(crate) type_cache: ImportCache,
  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  pub(crate) cached_types: RwOption<Vec<TypeDefinition>>,
  #[builder(setter(strip_option), default)]
  pub(crate) package: Option<PackageConfig>,
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

  /// Set the name of the component
  pub fn set_name(&mut self, name: String) {
    self.name = name;
  }

  /// Get the package files
  pub fn package_files(&self) -> Assets<AssetReference> {
    self.package.assets()
  }

  /// Get the configuration item a binding points to.
  #[must_use]
  pub fn resolve_binding(&self, name: &str) -> Option<ConfigurationItem> {
    if let Some(import) = self.import.get(name) {
      if let ImportDefinition::Component(component) = &import.kind {
        return Some(ConfigurationItem::Component(component));
      }
    }
    if let Some(resource) = self.resources.get(name) {
      return Some(ConfigurationItem::Resource(&resource.kind));
    }
    None
  }

  /// Returns a function that resolves a binding to a configuration item.
  #[must_use]
  pub fn resolver(&self) -> Box<Resolver> {
    make_resolver(self.import.clone(), self.resources.clone())
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

  /// Return the metadata of the component.
  #[must_use]
  pub fn metadata(&self) -> Option<&config::Metadata> {
    self.metadata.as_ref()
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

  pub fn into_v1_yaml(self) -> Result<String> {
    let v1_manifest: v1::AppConfiguration = self.try_into()?;
    Ok(serde_yaml::to_string(&v1_manifest).unwrap())
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
  pub fn try_component(&self) -> std::result::Result<ComponentDefinition, ReferenceError> {
    match self {
      Self::Component(c) => Ok(c.clone()),
      _ => Err(ReferenceError::Component),
    }
  }
  /// Get the resource definition or return an error.
  pub fn try_resource(&self) -> std::result::Result<ResourceDefinition, ReferenceError> {
    match self {
      Self::Resource(c) => Ok(c.clone()),
      _ => Err(ReferenceError::Resource),
    }
  }
}
