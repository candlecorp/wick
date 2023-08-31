#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
pub(super) mod triggers;

use asset_container::{AssetManager, Assets};
use tracing::trace;
use wick_asset_reference::{AssetReference, FetchOptions};
use wick_interface_types::TypeDefinition;
use wick_packet::{Entity, RuntimeConfig};

pub use self::triggers::*;
use super::common::component_definition::ComponentDefinition;
use super::common::package_definition::PackageConfig;
use super::components::TypesComponent;
use super::import_cache::{setup_cache, ImportCache};
use super::{ImportBinding, ImportDefinition};
use crate::config::common::resources::*;
use crate::config::template_config::Renderable;
use crate::error::{ManifestError, ReferenceError};
use crate::lockdown::{validate_resource, FailureKind, Lockdown, LockdownError};
use crate::utils::{make_resolver, resolve, RwOption};
use crate::{config, ExpandImports, Resolver, Result};

#[derive(
  Debug,
  Clone,
  Default,
  derive_builder::Builder,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(AssetReference))]
#[builder(
  setter(into),
  build_fn(name = "build_internal", private, error = "crate::error::BuilderError")
)]
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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) source: Option<PathBuf>,

  #[builder(setter(strip_option), default)]
  /// The metadata for the application.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) metadata: Option<config::Metadata>,

  #[builder(setter(strip_option), default)]
  /// The package configuration for this application.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) package: Option<PackageConfig>,

  #[builder(default)]
  /// The components that make up the application.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) import: Vec<ImportBinding>,

  #[builder(default)]
  /// Any resources this application defines.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) resources: Vec<ResourceBinding>,

  #[builder(default)]
  /// The triggers that initialize upon a `run` and make up the application.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) triggers: Vec<TriggerDefinition>,

  #[asset(skip)]
  #[doc(hidden)]
  #[builder(default)]
  #[property(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) root_config: Option<RuntimeConfig>,

  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  /// The environment this configuration has access to.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) env: Option<HashMap<String, String>>,

  #[asset(skip)]
  #[builder(setter(skip))]
  #[property(skip)]
  #[doc(hidden)]
  #[serde(skip)]
  pub(crate) type_cache: ImportCache,

  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) options: Option<FetchOptions>,

  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  #[doc(hidden)]
  #[serde(skip)]
  pub(crate) cached_types: RwOption<Vec<TypeDefinition>>,
}

impl AppConfiguration {
  /// Fetch/cache anything critical to the first use of this configuration.
  pub(crate) async fn setup_cache(&self, options: FetchOptions) -> Result<()> {
    setup_cache(
      &self.type_cache,
      self.import.iter(),
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

  /// Resolve an imported type by name.
  #[must_use]
  pub fn resolve_type(&self, name: &str) -> Option<TypeDefinition> {
    self
      .cached_types
      .read()
      .as_ref()
      .and_then(|types| types.iter().find(|t| t.name() == name).cloned())
  }

  /// Get the configuration item a binding points to.

  pub fn resolve_binding(&self, name: &str) -> Result<OwnedConfigurationItem> {
    resolve(name, &self.import, &self.resources)
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

  /// Return the version of the application.
  #[must_use]
  pub fn version(&self) -> Option<&str> {
    self.metadata.as_ref().map(|m| m.version.as_str())
  }

  /// Add a resource to the application configuration.
  pub fn add_resource(&mut self, name: impl AsRef<str>, resource: ResourceDefinition) {
    self.resources.push(ResourceBinding::new(name.as_ref(), resource));
  }

  /// Add a component to the application configuration.
  pub fn add_import(&mut self, name: impl AsRef<str>, import: ImportDefinition) {
    self.import.push(ImportBinding::new(name.as_ref(), import));
  }

  /// Generate V1 configuration yaml from this configuration.
  #[cfg(feature = "v1")]
  pub fn into_v1_yaml(self) -> Result<String> {
    let v1_manifest: crate::v1::AppConfiguration = self.try_into()?;
    Ok(serde_yaml::to_string(&v1_manifest).unwrap())
  }

  /// Initialize the configuration with the given environment variables.
  pub(super) fn initialize(&mut self) -> Result<&Self> {
    let root_config = self.root_config.as_ref();
    let source = self.source().map(std::path::Path::to_path_buf);

    trace!(
      source = ?source,
      num_resources = self.resources.len(),
      num_imports = self.import.len(),
      ?root_config,
      "initializing app resources"
    );
    let env = self.env.as_ref();

    let mut bindings = Vec::new();
    for (i, trigger) in self.triggers.iter_mut().enumerate() {
      trigger.expand_imports(&mut bindings, i)?;
    }
    self.import.extend(bindings);

    self.resources.render_config(source.as_deref(), root_config, env)?;
    self.import.render_config(source.as_deref(), root_config, env)?;
    self.triggers.render_config(source.as_deref(), root_config, env)?;

    Ok(self)
  }

  /// Validate this configuration is good.
  pub fn validate(&self) -> Result<()> {
    /* placeholder */
    Ok(())
  }
}

impl Lockdown for AppConfiguration {
  fn lockdown(
    &self,
    _id: Option<&str>,
    lockdown: &config::LockdownConfiguration,
  ) -> std::result::Result<(), LockdownError> {
    let mut errors = Vec::new();
    let id = Entity::LOCAL;

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

/// A configuration item
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub enum ConfigurationItem<'a> {
  /// A component definition.
  Component(&'a ComponentDefinition),
  /// A component definition.
  Types(&'a TypesComponent),
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

  /// Get the types definition or return an error.
  pub fn try_types(&self) -> std::result::Result<&'a TypesComponent, ReferenceError> {
    match self {
      Self::Types(c) => Ok(c),
      _ => Err(ReferenceError::Types),
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
  pub fn try_component(self) -> Result<ComponentDefinition> {
    match self {
      Self::Component(c) => Ok(c),
      _ => Err(ManifestError::Reference(ReferenceError::Component)),
    }
  }
  /// Get the resource definition or return an error.
  pub fn try_resource(self) -> Result<ResourceDefinition> {
    match self {
      Self::Resource(c) => Ok(c),
      _ => Err(ManifestError::Reference(ReferenceError::Resource)),
    }
  }
}

impl AppConfigurationBuilder {
  /// Build the configuration.
  pub fn build(&self) -> Result<AppConfiguration> {
    let config = self.clone();
    let config = config.build_internal()?;
    config.validate()?;
    Ok(config)
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;

  use super::*;
  use crate::config::components::ManifestComponentBuilder;
  use crate::config::{Codec, ComponentOperationExpressionBuilder, LiquidJsonConfig, MiddlewareBuilder};

  #[test]
  fn test_trigger_render() -> Result<()> {
    let op = ComponentOperationExpressionBuilder::default()
      .component(ComponentDefinition::Manifest(
        ManifestComponentBuilder::default()
          .reference("this/that:0.0.1")
          .build()?,
      ))
      .config(LiquidJsonConfig::try_from(
        json!({"op_config_field": "{{ctx.env.CARGO_MANIFEST_DIR}}"}),
      )?)
      .name("test")
      .build()?;
    let trigger = HttpTriggerConfigBuilder::default()
      .resource("URL")
      .routers(vec![HttpRouterConfig::RawRouter(RawRouterConfig {
        path: "/".to_owned(),
        middleware: Some(
          MiddlewareBuilder::default()
            .request(vec![op.clone()])
            .response(vec![op.clone()])
            .build()?,
        ),
        codec: Some(Codec::Json),
        operation: op,
      })])
      .build()?;
    let mut config = AppConfigurationBuilder::default()
      .name("test")
      .resources(vec![ResourceBinding::new("PORT", TcpPort::new("0.0.0.0", 90))])
      .triggers(vec![TriggerDefinition::Http(trigger)])
      .build()?;

    config.env = Some(std::env::vars().collect());
    config.root_config = Some(json!({"config_val": "from_config"}).try_into()?);

    config.initialize()?;

    let TriggerDefinition::Http(mut trigger) = config.triggers.pop().unwrap() else {
      unreachable!();
    };

    let HttpRouterConfig::RawRouter(mut router) = trigger.routers.pop().unwrap() else {
      unreachable!();
    };

    let cargo_manifest_dir = json!(env!("CARGO_MANIFEST_DIR"));

    let config = router.operation.config.take().unwrap().value.unwrap();
    assert_eq!(config.get("op_config_field"), Some(&cargo_manifest_dir));

    let mut mw = router.middleware.take().unwrap();

    let mw_req_config = mw.request[0].config.take().unwrap().value.unwrap();
    assert_eq!(mw_req_config.get("op_config_field"), Some(&cargo_manifest_dir));

    let mw_res_config = mw.response[0].config.take().unwrap().value.unwrap();
    assert_eq!(mw_res_config.get("op_config_field"), Some(&cargo_manifest_dir));

    Ok(())
  }
}
