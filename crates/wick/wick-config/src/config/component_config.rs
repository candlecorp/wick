mod composite;
mod wasm;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use asset_container::{AssetManager, Assets};
pub use composite::*;
use config::{ComponentImplementation, ComponentKind};
pub use wasm::*;
use wick_asset_reference::{AssetReference, FetchOptions};
use wick_interface_types::{ComponentMetadata, ComponentSignature, OperationSignature, TypeDefinition};

use super::common::package_definition::PackageConfig;
use super::{make_resolver, ImportBinding};
use crate::app_config::ResourceBinding;
use crate::common::BoundInterface;
use crate::import_cache::{setup_cache, ImportCache};
use crate::utils::RwOption;
use crate::{config, v1, Error, Resolver, Result};

#[derive(Debug, Default, Clone, Builder, derive_asset_container::AssetManager)]
#[builder(derive(Debug))]
#[asset(asset(AssetReference))]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct ComponentConfiguration {
  #[builder(default = "ComponentImplementation::Composite(CompositeComponentImplementation::default())")]
  pub(crate) component: ComponentImplementation,
  #[asset(skip)]
  #[builder(setter(into, strip_option), default)]
  pub name: Option<String>,
  #[asset(skip)]
  #[builder(setter(into, strip_option), default)]
  pub(crate) source: Option<PathBuf>,
  #[asset(skip)]
  #[builder(default)]
  pub(crate) types: Vec<TypeDefinition>,
  #[builder(default)]
  pub(crate) import: HashMap<String, ImportBinding>,
  #[asset(skip)]
  #[builder(default)]
  pub(crate) requires: HashMap<String, BoundInterface>,
  #[asset(skip)]
  #[builder(default)]
  pub(crate) resources: HashMap<String, ResourceBinding>,
  #[asset(skip)]
  #[builder(default)]
  pub(crate) host: config::HostConfig,
  #[asset(skip)]
  #[builder(default)]
  pub(crate) labels: HashMap<String, String>,
  #[asset(skip)]
  #[builder(default)]
  pub(crate) tests: Vec<config::TestCase>,
  #[asset(skip)]
  #[builder(default)]
  pub(crate) metadata: Option<config::Metadata>,
  #[asset(skip)]
  #[builder(setter(skip))]
  pub(crate) type_cache: ImportCache,
  #[asset(skip)]
  #[builder(setter(skip))]
  pub(crate) cached_types: RwOption<Vec<TypeDefinition>>,
  #[builder(default)]
  pub package: Option<PackageConfig>,
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

  /// Get the package files
  #[must_use]
  pub fn package_files(&self) -> Option<Assets<AssetReference>> {
    // should return empty vec if package is None
    self.package.as_ref().map(|p| p.assets())
  }

  #[must_use]
  pub fn operation_signatures(&self) -> Vec<OperationSignature> {
    match &self.component {
      ComponentImplementation::Composite(c) => c.operation_signatures(),
      ComponentImplementation::Wasm(c) => c.operation_signatures(),
      _ => unimplemented!(),
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
  pub fn set_source(&mut self, source: &Path) {
    // Source is a file, so our baseurl needs to be the parent directory.
    // Remove the trailing filename from source.
    if source.is_dir() {
      self.set_baseurl(source);
      self.source = Some(source.to_path_buf());
    } else {
      let mut s = source.to_path_buf();
      s.pop();

      self.set_baseurl(&s);
      self.source = Some(s);
    }
  }

  /// Returns a function that resolves a binding to a configuration item.
  #[must_use]
  pub fn resolver(&self) -> Box<Resolver> {
    let imports = self.import.clone();
    let resources = self.resources.clone();

    make_resolver(imports, resources)
  }

  /// Get the imports defined in this component.
  #[must_use]
  pub fn imports(&self) -> &HashMap<String, ImportBinding> {
    &self.import
  }

  /// Returns an [ImportBinding] if it exists in the configuration.
  #[must_use]
  pub fn get_import(&self, name: &str) -> Option<&ImportBinding> {
    self.import.get(name)
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
    self.metadata.clone().unwrap_or_default()
  }

  /// Return the underlying version of the source manifest.
  #[must_use]
  pub fn source(&self) -> Option<&Path> {
    self.source.as_deref()
  }

  /// Return the types defined in this component.
  pub fn types(&self) -> Result<Vec<TypeDefinition>> {
    self.cached_types.read().as_ref().map_or_else(
      || {
        if self.import.is_empty() {
          Ok(self.types.clone())
        } else {
          Err(Error::TypesNotFetched)
        }
      },
      |types| Ok(types.clone()),
    )
  }

  /// Return the requirements defined in this component.
  #[must_use]
  pub fn requires(&self) -> &HashMap<String, BoundInterface> {
    &self.requires
  }

  /// Fetch/cache anything critical to the first use of this configuration.
  pub(crate) async fn setup_cache(&self, options: FetchOptions) -> Result<()> {
    setup_cache(
      &self.type_cache,
      self.import.values(),
      &self.cached_types,
      self.types.clone(),
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
      name: self.name().clone().unwrap_or_else(||self.component.default_name().to_owned()),
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

impl ComponentConfigurationBuilder {
  #[must_use]
  pub fn from_base(config: ComponentConfiguration) -> Self {
    let mut this = Self::default();
    this
      .component(config.component)
      .host(config.host)
      .labels(config.labels)
      .tests(config.tests)
      .types(config.types)
      .requires(config.requires)
      .resources(config.resources)
      .metadata(config.metadata)
      .import(config.import);

    if let Some(name) = config.name {
      this.name(name);
    }
    if let Some(source) = config.source {
      this.source(source);
    }

    this
  }

  pub fn add_import(&mut self, import: ImportBinding) {
    if let Some(imports) = &mut self.import {
      imports.insert(import.id.clone(), import);
    } else {
      let mut imports = HashMap::new();
      imports.insert(import.id.clone(), import);
      self.import = Some(imports);
    }
  }

  pub fn add_resource(&mut self, resource: ResourceBinding) {
    if let Some(r) = &mut self.resources {
      r.insert(resource.id.clone(), resource);
    } else {
      let mut r = HashMap::new();
      r.insert(resource.id.clone(), resource);
      self.resources = Some(r);
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

impl From<config::OperationSignature> for OperationSignature {
  fn from(value: config::OperationSignature) -> Self {
    Self {
      name: value.name,
      inputs: value.inputs,
      outputs: value.outputs,
    }
  }
}
