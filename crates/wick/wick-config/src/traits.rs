use crate::config::ImportDefinition;

/// The [ExpandImports] trait is implemented by entities that may or may not need to alter the imported components.
#[cfg(feature = "config")]
pub trait ExpandImports {
  /// The type of error that may be returned when expanding imports.
  type Error;
  /// Expand imports with any inline definitions.
  fn expand_imports(
    &mut self,
    bindings: &mut Vec<crate::config::Binding<ImportDefinition>>,
    index: usize,
  ) -> Result<(), Self::Error>;
}
