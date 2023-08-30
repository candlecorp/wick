#![allow(missing_docs)] // delete when we move away from the `property` crate.
mod composite;
mod wasm;
use std::path::{Path, PathBuf};

use asset_container::{AssetManager, Assets};
pub use composite::*;
use config::{ComponentImplementation, ComponentKind};
use tracing::trace;
pub use wasm::*;
use wick_asset_reference::{AssetReference, FetchOptions};
use wick_interface_types::{ComponentMetadata, ComponentSignature, Field, OperationSignature, TypeDefinition};
use wick_packet::{Entity, RuntimeConfig};

use super::common::package_definition::PackageConfig;
use super::import_cache::{setup_cache, ImportCache};
use super::{ImportBinding, TestConfiguration};
use crate::config::template_config::Renderable;
use crate::config::{BoundInterface, ResourceBinding};
use crate::lockdown::{validate_resource, FailureKind, Lockdown, LockdownError};
use crate::utils::{make_resolver, RwOption};
use crate::{config, Error, Resolver, Result};

#[derive(
  Debug,
  Default,
  Clone,
  derive_builder::Builder,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
)]
#[builder(
  derive(Debug),
  setter(into),
  build_fn(name = "build_internal", private, error = "crate::error::BuilderError")
)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(AssetReference))]
#[must_use]
/// A Wick component configuration.
///
/// A component configuration defines a wick component and its operations along with its dependencies
/// immediate dependencies and any dependencies that it requires be provided by the user.
pub struct ComponentConfiguration {
  #[asset(skip)]
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// The name of the component configuration.
  pub(crate) name: Option<String>,

  /// The component implementation.
  pub(crate) component: ComponentImplementation,

  #[asset(skip)]
  #[builder(setter(strip_option), default)]
  #[property(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  /// The source (i.e. url or file on disk) of the configuration.
  pub(crate) source: Option<PathBuf>,

  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  /// Any types referenced or exported by this component.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<TypeDefinition>,

  #[builder(default)]
  /// Any imports this component makes available to its implementation.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) import: Vec<ImportBinding>,

  #[asset(skip)]
  #[builder(default)]
  /// Any components or resources that must be provided to this component upon instantiation.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) requires: Vec<BoundInterface>,

  #[builder(default)]
  /// Any resources this component defines.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) resources: Vec<ResourceBinding>,

  #[asset(skip)]
  #[builder(default)]
  /// The configuration to use when running this component as a microservice.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) host: Option<config::HostConfig>,

  #[asset(skip)]
  #[builder(default)]
  /// Any embedded test cases that should be run against this component.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) tests: Vec<TestConfiguration>,

  #[asset(skip)]
  #[builder(default)]
  /// The metadata for this component.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) metadata: Option<config::Metadata>,

  #[builder(default)]
  /// The package configuration for this component.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) package: Option<PackageConfig>,

  #[asset(skip)]
  #[doc(hidden)]
  #[property(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) root_config: Option<RuntimeConfig>,

  #[asset(skip)]
  #[builder(setter(skip))]
  #[property(skip)]
  #[doc(hidden)]
  #[serde(skip)]
  pub(crate) type_cache: ImportCache,

  #[asset(skip)]
  #[builder(setter(skip))]
  #[property(skip)]
  #[doc(hidden)]
  #[serde(skip)]
  pub(crate) cached_types: RwOption<Vec<TypeDefinition>>,
}

impl ComponentConfiguration {
  /// Unwrap the inner composite component implementation or return an error.
  pub fn try_composite(&self) -> Result<&CompositeComponentImplementation> {
    match &self.component {
      ComponentImplementation::Composite(c) => Ok(c),
      _ => Err(Error::UnexpectedComponentType(
        ComponentKind::Composite,
        self.component.kind(),
      )),
    }
  }

  /// Unwrap the inner wasm component implementation or return an error.
  pub fn try_wasm(&self) -> Result<&WasmComponentImplementation> {
    match &self.component {
      ComponentImplementation::Wasm(c) => Ok(c),
      _ => Err(Error::UnexpectedComponentType(
        ComponentKind::Wasm,
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

  /// Set the source location of the configuration.
  pub fn set_source(&mut self, source: &Path) {
    let source = source.to_path_buf();
    self.source = Some(source);
  }

  pub(super) fn update_baseurls(&self) {
    #[allow(clippy::expect_used)]
    let mut source = self.source.clone().expect("No source set for this configuration");
    // Source is (should be) a file, so pop the filename before setting the baseurl.
    if !source.is_dir() {
      source.pop();
    }

    self.set_baseurl(&source);
  }

  /// Returns a function that resolves a binding to a configuration item.
  #[must_use]
  pub fn resolver(&self) -> Box<Resolver> {
    trace!("creating resolver for component {:?}", self.name());
    make_resolver(
      self.import.clone(),
      self.resources.clone(),
      self.root_config.clone(),
      None,
    )
  }

  /// Get the kind of this component implementation.
  pub fn kind(&self) -> ComponentKind {
    self.component.kind()
  }

  /// Determine if the configuration allows for fetching artifacts with the :latest tag.
  #[must_use]
  pub fn allow_latest(&self) -> bool {
    self.host.as_ref().map_or(false, |v| v.allow_latest)
  }

  /// Return the list of insecure registries defined in the manifest
  #[must_use]
  pub fn insecure_registries(&self) -> Option<&[String]> {
    self.host.as_ref().map(|v| v.insecure_registries.as_ref())
  }

  /// Return the version of the component.
  #[must_use]
  pub fn version(&self) -> Option<&str> {
    self.metadata.as_ref().map(|m| m.version.as_str())
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

  /// Get a mutable reference to the type definitions for this component.
  pub fn types_mut(&mut self) -> &mut Vec<TypeDefinition> {
    &mut self.types
  }

  /// Fetch/cache anything critical to the first use of this configuration.
  pub(crate) async fn setup_cache(&self, options: FetchOptions) -> Result<()> {
    setup_cache(
      &self.type_cache,
      self.import.iter(),
      &self.cached_types,
      self.types.clone(),
      options,
    )
    .await
  }

  #[must_use]
  pub fn config(&self) -> &[Field] {
    match &self.component {
      ComponentImplementation::Composite(c) => &c.config,
      ComponentImplementation::Wasm(c) => &c.config,
      ComponentImplementation::Sql(c) => &c.config,
      ComponentImplementation::HttpClient(c) => &c.config,
    }
  }

  #[must_use]
  pub fn root_config(&self) -> Option<&RuntimeConfig> {
    self.root_config.as_ref()
  }

  /// Get the component signature for this configuration.
  pub fn signature(&self) -> Result<ComponentSignature> {
    let mut sig = wick_interface_types::component! {
      name: self.name().cloned().unwrap_or_else(||self.component.default_name().to_owned()),
      version: self.version(),
      operations: self.component.operation_signatures(),
    };
    sig.config = self.config().to_vec();
    sig.types = self.types()?;
    Ok(sig)
  }

  /// Return the V1 yaml representation of this configuration.
  #[cfg(feature = "v1")]
  pub fn into_v1_yaml(self) -> Result<String> {
    let v1_manifest: crate::v1::ComponentConfiguration = self.try_into()?;
    Ok(serde_yaml::to_string(&v1_manifest).unwrap())
  }

  /// Initialize the configuration.
  pub(super) fn initialize(&mut self) -> Result<&Self> {
    // This pre-renders the component config's resources without access to the environment.
    let root_config = self.root_config.as_ref();
    trace!(
      num_resources = self.resources.len(),
      num_imports = self.import.len(),
      ?root_config,
      "initializing component"
    );

    self.resources.render_config(root_config, None)?;
    self.import.render_config(root_config, None)?;

    Ok(self)
  }

  /// Validate this configuration is good.
  pub fn validate(&self) -> Result<()> {
    wick_packet::validation::expect_configuration_matches(
      self.source().map_or("<unknown>", |p| p.to_str().unwrap_or("<invalid>")),
      self.root_config.as_ref(),
      self.config(),
    )
    .map_err(Error::ConfigurationInvalid)?;
    Ok(())
  }
}

impl Lockdown for ComponentConfiguration {
  fn lockdown(
    &self,
    id: Option<&str>,
    lockdown: &config::LockdownConfiguration,
  ) -> std::result::Result<(), LockdownError> {
    let mut errors = Vec::new();
    let Some(id) = id else {
      return Err(LockdownError::new(vec![FailureKind::General(
        "missing component id".into(),
      )]));
    };

    if id == Entity::LOCAL {
      return Err(LockdownError::new(vec![FailureKind::General(format!(
        "invalid component id: {}",
        Entity::LOCAL
      ))]));
    }

    for resource in self.resources.iter() {
      if let Err(e) = validate_resource(id, &(resource.into()), lockdown) {
        errors.push(FailureKind::Failed(Box::new(e)));
      }
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(LockdownError::new(errors))
    }
  }
}

impl ComponentConfigurationBuilder {
  #[must_use]
  /// Initialize a new component configuration builder from an existing configuration.
  pub fn from_base(config: ComponentConfiguration) -> Self {
    Self {
      name: Some(config.name),
      component: Some(config.component),
      source: None,
      types: Some(config.types),
      import: Some(config.import),
      requires: Some(config.requires),
      resources: Some(config.resources),
      host: Some(config.host),
      tests: Some(config.tests),
      metadata: Some(config.metadata),
      package: Some(config.package),
      root_config: Some(config.root_config),
      type_cache: std::marker::PhantomData,
      cached_types: std::marker::PhantomData,
    }
  }

  /// Add an imported component to the builder.
  pub fn add_import(&mut self, import: ImportBinding) {
    if let Some(imports) = &mut self.import {
      imports.push(import);
    } else {
      self.import = Some(vec![import]);
    }
  }

  /// Add an imported resource to the builder.
  pub fn add_resource(&mut self, resource: ResourceBinding) {
    if let Some(r) = &mut self.resources {
      r.push(resource);
    } else {
      self.resources = Some(vec![resource]);
    }
  }

  /// Build the configuration.
  pub fn build(self) -> Result<ComponentConfiguration> {
    let config = self.build_internal()?;
    config.validate()?;
    Ok(config)
  }
}

impl From<config::Metadata> for ComponentMetadata {
  fn from(value: config::Metadata) -> Self {
    Self {
      version: Some(value.version),
    }
  }
}

impl From<config::OperationDefinition> for OperationSignature {
  fn from(value: config::OperationDefinition) -> Self {
    Self {
      name: value.name,
      config: value.config,
      inputs: value.inputs,
      outputs: value.outputs,
    }
  }
}
