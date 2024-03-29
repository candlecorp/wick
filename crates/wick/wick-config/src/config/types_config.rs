#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use asset_container::{AssetManager, Assets};
use wick_asset_reference::AssetReference;
use wick_interface_types::{OperationSignatures, TypeDefinition};
use wick_packet::RuntimeConfig;

use super::common::package_definition::PackageConfig;
use super::components::ComponentConfig;
use super::template_config::Renderable;
use super::OperationDefinition;
use crate::config;
use crate::error::ManifestError;

#[derive(
  Debug, Clone, derive_builder::Builder, derive_asset_container::AssetManager, property::Property, serde::Serialize,
)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(AssetReference))]
#[must_use]
/// A Wick types configuration.
///
/// A types configuration is a collection of shareable types and operation signatures used to generated
/// code for components and other types.
pub struct TypesConfiguration {
  /// The name of the types configuration.
  #[asset(skip)]
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,

  /// The source (i.e. url or file on disk) of the configuration.
  #[asset(skip)]
  #[property(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) source: Option<PathBuf>,

  /// Any metadata associated with the configuration.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) metadata: Option<config::Metadata>,

  /// A list of types defined in this configuration.
  #[asset(skip)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<TypeDefinition>,

  /// A list of operation signatures defined in this configuration.
  #[asset(skip)]
  #[property(skip)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<OperationDefinition>,

  /// The package configuration for this configuration.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) package: Option<PackageConfig>,
}

impl TypesConfiguration {
  /// Get a type by name
  #[must_use]
  pub fn get_type(&self, name: &str) -> Option<&TypeDefinition> {
    self.types.iter().find(|t| t.name() == name)
  }

  /// Return the version of the application.
  #[must_use]
  pub fn version(&self) -> Option<&str> {
    self.metadata.as_ref().map(|m| m.version.as_str())
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

  /// Return the environment variables for this configuration.
  #[must_use]
  pub const fn env(&self) -> Option<&HashMap<String, String>> {
    None
  }

  /// Validate this configuration is good.
  #[allow(clippy::missing_const_for_fn)]
  pub fn validate(&self) -> Result<(), ManifestError> {
    /* placeholder */
    Ok(())
  }
}

impl OperationSignatures for TypesConfiguration {
  fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.clone().into_iter().map(Into::into).collect()
  }
}

impl ComponentConfig for TypesConfiguration {
  type Operation = OperationDefinition;

  fn operations(&self) -> &[Self::Operation] {
    &self.operations
  }

  fn operations_mut(&mut self) -> &mut Vec<Self::Operation> {
    &mut self.operations
  }
}

impl Renderable for TypesConfiguration {
  fn render_config(
    &mut self,
    _source: Option<&Path>,
    _root_config: Option<&RuntimeConfig>,
    _env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    Ok(())
  }
}
