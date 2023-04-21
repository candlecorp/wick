mod composite;
mod wasm;
use std::collections::HashMap;

use asset_container::AssetManager;
pub use composite::CompositeComponentImplementation;
use config::{ComponentImplementation, ComponentKind};
pub use wasm::{OperationSignature, WasmComponentImplementation};
use wick_asset_reference::{AssetReference, FetchOptions};
use wick_interface_types::{ComponentMetadata, ComponentSignature, TypeDefinition};

use super::{make_resolver, ImportBinding};
use crate::app_config::ResourceBinding;
use crate::import_cache::{setup_cache, ImportCache};
use crate::utils::RwOption;
use crate::{config, v1, Error, Resolver, Result};

#[derive(Debug, Default, Clone, derive_asset_container::AssetManager)]
#[asset(AssetReference)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct ComponentConfiguration {
  #[asset(skip)]
  pub name: Option<String>,
  #[asset(skip)]
  pub(crate) source: Option<String>,
  #[asset(skip)]
  pub(crate) resources: HashMap<String, ResourceBinding>,
  #[asset(skip)]
  pub(crate) host: config::HostConfig,
  #[asset(skip)]
  pub(crate) labels: HashMap<String, String>,
  #[asset(skip)]
  pub(crate) tests: Vec<config::TestCase>,
  #[asset(skip)]
  pub(crate) metadata: Option<config::Metadata>,
  pub(crate) component: ComponentImplementation,
  #[asset(skip)]
  pub(crate) type_cache: ImportCache,
  #[asset(skip)]
  pub(crate) cached_types: RwOption<Vec<TypeDefinition>>,
}

impl ComponentConfiguration {
  pub fn try_composite(&self) -> Result<&CompositeComponentImplementation> {
    match &self.component {
      ComponentImplementation::Composite(c) => Ok(c),
      _ => Err(Error::UnexpectedComponentType(
        ComponentKind::Composite,
        self.component.kind(),
      )),
    }
  }

  pub fn try_wasm(&self) -> Result<&WasmComponentImplementation> {
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

  /// Returns a function that resolves a binding to a configuration item.
  #[must_use]
  pub fn resolver(&self) -> Box<Resolver> {
    let (imports, resources) = match self.component {
      ComponentImplementation::Wasm(ref c) => (c.import.clone(), self.resources.clone()),
      ComponentImplementation::Composite(ref c) => (c.import.clone(), self.resources.clone()),
    };
    make_resolver(imports, resources)
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
  pub fn types(&self) -> Result<Vec<TypeDefinition>> {
    self.cached_types.read().as_ref().map_or_else(
      || {
        if self.component.imports().is_empty() {
          Ok(self.component.types().to_vec())
        } else {
          Err(Error::TypesNotFetched)
        }
      },
      |types| Ok(types.clone()),
    )
  }

  /// Fetch/cache anything critical to the first use of this configuration.
  pub(crate) async fn setup_cache(&self, options: FetchOptions) -> Result<()> {
    setup_cache(
      &self.type_cache,
      self.component.imports().values(),
      &self.cached_types,
      self.component.types().to_vec(),
      options,
    )
    .await
  }

  /// Return the resources defined in this component.
  #[must_use]
  pub fn resources(&self) -> &HashMap<String, ResourceBinding> {
    &self.resources
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

  /// Get the component signature for this configuration.
  pub fn signature(&self) -> Result<ComponentSignature> {
    let mut sig = wick_interface_types::component! {
      name: self.name().clone().unwrap_or_else(||"".to_owned()),
      version: self.version(),
      operations: self.component.operation_signatures(),
    };
    sig.types = self.types()?;
    Ok(sig)
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
  resources: HashMap<String, ResourceBinding>,
}

impl Default for ComponentConfigurationBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl ComponentConfigurationBuilder {
  /// Create a new [ComponentConfigurationBuilder].
  pub fn new() -> Self {
    let component = CompositeComponentImplementation::default();
    Self {
      component: ComponentImplementation::Composite(component),
      resources: Default::default(),
      name: Default::default(),
      source: Default::default(),
      host: Default::default(),
      labels: Default::default(),
      tests: Default::default(),
      metadata: Default::default(),
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
      resources: Default::default(),
    }
  }

  pub fn add_import(mut self, import: ImportBinding) -> Self {
    self.component = match self.component {
      ComponentImplementation::Composite(c) => ComponentImplementation::Composite(c.add_import(import)),
      ComponentImplementation::Wasm(_) => panic!("Can not add imports to anything but a Composite component"),
    };
    self
  }

  pub fn add_resource(mut self, resource: ResourceBinding) -> Self {
    self.resources.insert(resource.id.clone(), resource);
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
      resources: self.resources,
      cached_types: Default::default(),
      type_cache: Default::default(),
    }
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
