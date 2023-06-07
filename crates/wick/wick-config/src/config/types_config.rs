use std::collections::HashMap;
use std::path::{Path, PathBuf};

use asset_container::{AssetManager, Assets};
use wick_asset_reference::AssetReference;
use wick_interface_types::TypeDefinition;

use super::common::package_definition::PackageConfig;
use super::OperationSignature;
use crate::config;

#[derive(Debug, Clone, Builder, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(AssetReference))]
#[must_use]
pub struct TypesConfiguration {
  #[asset(skip)]
  #[builder(setter(strip_option), default)]
  pub(crate) name: Option<String>,
  #[asset(skip)]
  #[property(skip)]
  pub(crate) source: Option<PathBuf>,
  #[asset(skip)]
  #[builder(default)]
  pub(crate) metadata: Option<config::Metadata>,
  #[asset(skip)]
  pub(crate) types: Vec<TypeDefinition>,
  #[asset(skip)]
  pub(crate) operations: HashMap<String, OperationSignature>,
  #[builder(default)]
  pub(crate) package: Option<PackageConfig>,
}

impl TypesConfiguration {
  /// Get the inner definitions, consuming the [TypesConfiguration].
  #[must_use]
  pub fn into_parts(self) -> (Vec<TypeDefinition>, HashMap<String, OperationSignature>) {
    (self.types, self.operations)
  }

  /// Get the types defined in this configuration, consuming the [TypesConfiguration].
  #[must_use]
  pub fn into_types(self) -> Vec<TypeDefinition> {
    self.types
  }

  /// Get the operations defined in this configuration, consuming the [TypesConfiguration].
  #[must_use]
  pub fn into_operations(self) -> HashMap<String, OperationSignature> {
    self.operations
  }

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
    let mut source = source.to_path_buf();
    self.source = Some(source.clone());
    // Source is (should be) a file, so pop the filename before setting the baseurl.
    if !source.is_dir() {
      source.pop();
    }
    self.set_baseurl(&source);
  }
}
