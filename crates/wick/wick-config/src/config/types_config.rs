use assets::AssetManager;
use url::Url;
use wick_interface_types::TypeDefinition;

#[derive(Debug, Clone, derive_assets::AssetManager)]
#[asset(crate::config::AssetReference)]
#[must_use]
pub struct TypesConfiguration {
  #[asset(skip)]
  pub(crate) source: Option<Url>,
  #[asset(skip)]
  pub(crate) types: Vec<TypeDefinition>,
}

impl TypesConfiguration {
  /// Get the types defined in this configuration.
  pub fn types(&self) -> &[TypeDefinition] {
    &self.types
  }

  /// Set the source location of the configuration.
  pub fn set_source(&mut self, source: Url) {
    // Source is a file, so our baseurl needs to be the parent directory.
    self.set_baseurl(source.join("./").unwrap().as_str());
    self.source = Some(source);
  }
}
