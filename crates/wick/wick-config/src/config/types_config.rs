use std::collections::HashMap;
use std::path::{Path, PathBuf};

use asset_container::AssetManager;
use wick_interface_types::TypeDefinition;

use super::OperationSignature;

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(crate::config::AssetReference))]
#[must_use]
pub struct TypesConfiguration {
  #[asset(skip)]
  pub(crate) source: Option<PathBuf>,
  #[asset(skip)]
  pub(crate) types: Vec<TypeDefinition>,
  #[asset(skip)]
  pub(crate) operations: HashMap<String, OperationSignature>,
}

impl TypesConfiguration {
  /// Get the types defined in this configuration.
  pub fn types(&self) -> &[TypeDefinition] {
    &self.types
  }

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

  /// Get the operations defined in this configuration.
  #[must_use]
  pub fn operations(&self) -> &HashMap<String, OperationSignature> {
    &self.operations
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
}
