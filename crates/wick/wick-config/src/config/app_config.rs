use std::collections::HashMap;
pub(super) mod resources;
pub(super) mod triggers;

use asset_container::AssetManager;

pub use self::resources::*;
pub use self::triggers::*;
use super::common::component_definition::ComponentDefinition;
use super::common::host_definition::HostConfig;
use super::BoundComponent;
use crate::error::ReferenceError;
use crate::{config, v1, Resolver, Result};

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(config::AssetReference)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct AppConfiguration {
  #[asset(skip)]
  pub name: String,
  #[asset(skip)]
  pub(crate) source: Option<String>,
  pub(crate) metadata: Option<config::Metadata>,
  pub(crate) import: HashMap<String, BoundComponent>,
  #[asset(skip)]
  pub(crate) resources: HashMap<String, BoundResource>,
  pub(crate) triggers: Vec<TriggerDefinition>,
  #[asset(skip)]
  pub(crate) host: HostConfig,
}

impl Default for AppConfiguration {
  fn default() -> Self {
    Self {
      name: "".to_owned(),
      source: None,
      metadata: None,
      host: HostConfig::default(),
      import: HashMap::new(),
      resources: HashMap::new(),
      triggers: vec![],
    }
  }
}

impl AppConfiguration {
  /// Get the configuration item a binding points to.
  #[must_use]
  pub fn resolve_binding(&self, name: &str) -> Option<ConfigurationItem> {
    if let Some(component) = self.import.get(name) {
      return Some(ConfigurationItem::Component(&component.kind));
    }
    if let Some(resource) = self.resources.get(name) {
      return Some(ConfigurationItem::Resource(&resource.kind));
    }
    None
  }

  /// Returns a function that resolves a binding to a configuration item.
  #[must_use]
  pub fn resolver(&self) -> Resolver {
    let imports = self.import.clone();
    let resources = self.resources.clone();
    Box::new(move |name| {
      if let Some(component) = imports.get(name) {
        return Some(OwnedConfigurationItem::Component(component.kind.clone()));
      }
      if let Some(resource) = resources.get(name) {
        return Some(OwnedConfigurationItem::Resource(resource.kind.clone()));
      }
      None
    })
  }

  /// Return the underlying version of the source manifest.
  #[must_use]
  pub fn source(&self) -> &Option<String> {
    &self.source
  }

  /// Set the source location of the configuration.
  pub fn set_source(&mut self, source: String) {
    // Source is a file, so our baseurl needs to be the parent directory.
    // Remove the trailing filename from source.
    if source.ends_with(std::path::MAIN_SEPARATOR) {
      self.set_baseurl(&source);
      self.source = Some(source);
    } else {
      let s = source.rfind('/').map_or(source.as_str(), |index| &source[..index]);

      self.set_baseurl(s);
      self.source = Some(s.to_owned());
    }
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn name(&self) -> String {
    self.name.clone()
  }

  /// Return the version of the application.
  #[must_use]
  pub fn version(&self) -> String {
    self.metadata.clone().map(|m| m.version).unwrap_or_default()
  }

  /// Return the metadata of the component.
  #[must_use]
  pub fn metadata(&self) -> config::Metadata {
    self.metadata.clone().unwrap()
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

  /// Add a resource to the application configuration.
  pub fn add_resource(&mut self, name: impl AsRef<str>, resource: ResourceDefinition) {
    self
      .resources
      .insert(name.as_ref().to_owned(), BoundResource::new(name.as_ref(), resource));
  }

  /// Add a component to the application configuration.
  pub fn add_import(&mut self, name: impl AsRef<str>, component: ComponentDefinition) {
    self
      .import
      .insert(name.as_ref().to_owned(), BoundComponent::new(name.as_ref(), component));
  }

  #[must_use]
  /// Get the application's triggers.
  pub fn triggers(&self) -> &Vec<TriggerDefinition> {
    &self.triggers
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
