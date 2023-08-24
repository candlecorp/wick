use crate::config;

/// The [TriggerConfiguration] trait is implemented by unknown configuration entities that may or may not need to alter the [AppConfiguration] they are running within.
pub trait ExpandImports {
  /// The type of error that may be returned when expanding imports.
  type Error;
  /// Expand imports with any inline definitions.
  fn expand_imports(&mut self, bindings: &mut Vec<config::ImportBinding>, index: usize) -> Result<(), Self::Error>;
}
