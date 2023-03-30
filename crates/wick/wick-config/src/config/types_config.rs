use assets::AssetManager;
use wick_interface_types::TypeDefinition;

#[derive(Debug, Clone, derive_assets::AssetManager)]
#[asset(crate::config::LocationReference)]
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
  pub fn set_source(&mut self, source: impl AsRef<str>) {
    self.source = Some(source.as_ref().to_owned());
    self.set_baseurl(source.as_ref());
  }
}
