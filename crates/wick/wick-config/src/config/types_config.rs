use asset_container::AssetManager;
use wick_interface_types::TypeDefinition;

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(crate::config::AssetReference)]
#[must_use]
pub struct TypesConfiguration {
  #[asset(skip)]
  pub(crate) source: Option<String>,
  #[asset(skip)]
  pub(crate) types: Vec<TypeDefinition>,
}

impl TypesConfiguration {
  /// Get the types defined in this configuration.
  pub fn types(&self) -> &[TypeDefinition] {
    &self.types
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
}
