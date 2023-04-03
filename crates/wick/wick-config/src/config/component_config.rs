mod composite;
mod wasm;
use std::collections::HashMap;

use assets::AssetManager;
pub use composite::CompositeComponentConfiguration;
use config::{ComponentImplementation, ComponentKind};
pub use wasm::{OperationSignature, WasmComponentConfiguration};
use wick_interface_types::{ComponentMetadata, ComponentSignature, ComponentVersion, TypeDefinition};

use super::BoundComponent;
use crate::{config, v1, Error, Result};

#[derive(Debug, Default, Clone, derive_assets::AssetManager)]
#[asset(config::AssetReference)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct ComponentConfiguration {
  #[asset(skip)]
  pub name: Option<String>,
  #[asset(skip)]
  pub(crate) source: Option<String>,
  #[asset(skip)]
  pub(crate) host: config::HostConfig,
  #[asset(skip)]
  pub(crate) labels: HashMap<String, String>,
  #[asset(skip)]
  pub(crate) tests: Vec<config::TestCase>,
  #[asset(skip)]
  pub(crate) metadata: Option<config::Metadata>,
  pub(crate) component: ComponentImplementation,
}

impl ComponentConfiguration {
  pub fn try_composite(&self) -> Result<&CompositeComponentConfiguration> {
    match &self.component {
      ComponentImplementation::Composite(c) => Ok(c),
      _ => Err(Error::UnexpectedComponentType(
        ComponentKind::Composite,
        self.component.kind(),
      )),
    }
  }

  pub fn try_wasm(&self) -> Result<&WasmComponentConfiguration> {
    match &self.component {
      ComponentImplementation::Wasm(c) => Ok(c),
      _ => Err(Error::UnexpectedComponentType(
        ComponentKind::Wasm,
        self.component.kind(),
      )),
    }
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

  /// Determine if the configuration allows for fetching artifacts with the :latest tag.
  pub fn host(&self) -> &config::HostConfig {
    &self.host
  }

  /// Determine if the configuration allows for fetching artifacts with the :latest tag.
  pub fn host_mut(&mut self) -> &mut config::HostConfig {
    &mut self.host
  }

  /// Get the configuration related to the specific [ComponentKind].
  pub fn component(&self) -> &ComponentImplementation {
    &self.component
  }

  /// Get the kind of this component implementation.
  pub fn kind(&self) -> ComponentKind {
    self.component.kind()
  }

  /// Determine if the configuration allows for fetching artifacts with the :latest tag.
  #[must_use]
  pub fn allow_latest(&self) -> bool {
    self.host.allow_latest
  }

  /// Return the list of insecure registries defined in the manifest
  #[must_use]
  pub fn insecure_registries(&self) -> &Vec<String> {
    &self.host.insecure_registries
  }

  /// Return the list of tests defined in the manifest.
  #[must_use]
  pub fn tests(&self) -> &[config::TestCase] {
    &self.tests
  }

  /// Return the version of the component.
  #[must_use]
  pub fn version(&self) -> String {
    self.metadata.clone().map(|m| m.version).unwrap_or_default()
  }

  /// Return the metadata of the component.
  #[must_use]
  pub fn metadata(&self) -> config::Metadata {
    self.metadata.clone().unwrap()
  }

  /// Return the underlying version of the source manifest.
  #[must_use]
  pub fn source(&self) -> &Option<String> {
    &self.source
  }

  /// Return the types defined in this component.
  pub fn types(&self) -> &[TypeDefinition] {
    self.component.types()
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn name(&self) -> &Option<String> {
    &self.name
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn labels(&self) -> &HashMap<String, String> {
    &self.labels
  }

  pub fn into_v1_yaml(self) -> Result<String> {
    let v1_manifest: v1::ComponentConfiguration = self.try_into()?;
    Ok(serde_yaml::to_string(&v1_manifest).unwrap())
  }
}

/// ComponentConfiguration builder.
#[derive(Debug, Clone)]
#[must_use]
pub struct ComponentConfigurationBuilder {
  name: Option<String>,
  source: Option<String>,
  host: config::HostConfig,
  labels: HashMap<String, String>,
  tests: Vec<config::TestCase>,
  component: ComponentImplementation,
  metadata: Option<config::Metadata>,
}

impl Default for ComponentConfigurationBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl ComponentConfigurationBuilder {
  /// Create a new [ComponentConfigurationBuilder].
  pub fn new() -> Self {
    let component = CompositeComponentConfiguration::default();
    Self {
      name: Default::default(),
      source: Default::default(),
      host: Default::default(),
      labels: Default::default(),
      tests: Default::default(),
      metadata: Default::default(),
      component: ComponentImplementation::Composite(component),
    }
  }

  /// Create a builder with an existing manifest as a base.
  pub fn with_base(definition: ComponentConfiguration) -> Self {
    Self {
      name: definition.name,
      source: definition.source,
      host: definition.host,
      labels: definition.labels,
      tests: definition.tests,
      metadata: definition.metadata,
      component: definition.component,
    }
  }

  pub fn add_import(mut self, import: BoundComponent) -> Self {
    self.component = match self.component {
      ComponentImplementation::Composite(c) => ComponentImplementation::Composite(c.add_import(import)),
      ComponentImplementation::Wasm(_) => panic!("Can not add imports to anything but a Composite component"),
    };
    self
  }

  /// Consume the [ComponentConfigurationBuilder] and return a [ComponentConfiguration].
  pub fn build(self) -> ComponentConfiguration {
    ComponentConfiguration {
      component: self.component,
      name: self.name,
      source: self.source,
      host: self.host,
      labels: self.labels,
      metadata: self.metadata,
      tests: self.tests,
    }
  }
}

impl TryFrom<ComponentConfiguration> for ComponentSignature {
  type Error = Error;
  fn try_from(value: ComponentConfiguration) -> Result<Self> {
    let c = match value.component {
      ComponentImplementation::Wasm(c) => Self {
        name: value.name,
        format: ComponentVersion::V1,
        metadata: value.metadata.map(|m| m.into()).unwrap_or_default(),
        wellknown: vec![],
        types: c.types().to_vec(),
        operations: c.operations.into_values().map(|o| o.into()).collect(),
        config: Default::default(),
      },
      ComponentImplementation::Composite(_) => todo!(),
    };
    Ok(c)
  }
}

impl From<config::Metadata> for ComponentMetadata {
  fn from(value: config::Metadata) -> Self {
    Self {
      version: Some(value.version),
    }
  }
}

impl From<OperationSignature> for wick_interface_types::OperationSignature {
  fn from(value: OperationSignature) -> Self {
    Self {
      name: value.name,
      inputs: value.inputs,
      outputs: value.outputs,
    }
  }
}
