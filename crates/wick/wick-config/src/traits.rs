/// The [ExpandImports] trait is implemented by entities that may or may not need to alter the imported components.
#[cfg(feature = "config")]
pub trait ExpandImports {
  /// The type of error that may be returned when expanding imports.
  type Error;
  /// Expand imports with any inline definitions.
  fn expand_imports(
    &mut self,
    bindings: &mut Vec<crate::config::Binding<ImportDefinition>>,
    bindings: &mut Vec<crate::config::Binding<crate::config::ImportDefinition>>,
    index: usize,
  ) -> Result<(), Self::Error>;
}

/// The [Imports] trait is implemented by configuration that can import other configuration.
#[cfg(feature = "config")]
pub trait Imports {
  /// Return a list of the imported configuration bindings.
  fn imports(&self) -> &[crate::config::Binding<crate::config::ImportDefinition>];
}

/// The [RootConfig] trait is implemented by configurations that have root-level runtime configuration.
#[cfg(feature = "config")]
pub trait RootConfig {
  /// Return the rendered [wick_packet::RuntimeConfig].
  fn root_config(&self) -> Option<&wick_packet::RuntimeConfig>;

  /// Set a rendered [wick_packet::RuntimeConfig].
  fn set_root_config(&mut self, config: Option<wick_packet::RuntimeConfig>);
}
