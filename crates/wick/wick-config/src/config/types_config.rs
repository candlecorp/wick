use wick_interface_types::TypeDefinition;

#[derive(Debug, Clone)]
#[must_use]
pub struct TypesConfiguration {
  pub(crate) source: Option<String>,
  pub(crate) types: Vec<TypeDefinition>,
}

impl TypesConfiguration {
  /// Get the types defined in this configuration.
  pub fn types(&self) -> &[TypeDefinition] {
    &self.types
  }
}
