use std::collections::HashMap;

use wick_interface_types::TypeDefinition;

use crate::config::common::component_definition::BoundComponent;
use crate::config::common::flow_definition::FlowOperation;

#[derive(Debug, Clone)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct CompositeComponentConfiguration {
  pub(crate) types: Vec<TypeDefinition>,
  pub(crate) import: HashMap<String, BoundComponent>,
  pub(crate) operations: HashMap<String, FlowOperation>,
}

impl CompositeComponentConfiguration {
  /// Add a [BoundComponent] to the configuration.
  pub fn add_import(mut self, name: impl AsRef<str>, component: BoundComponent) -> Self {
    self.import.insert(name.as_ref().to_owned(), component);
    self
  }

  /// Get the types used by this component.
  pub fn types(&self) -> &[TypeDefinition] {
    &self.types
  }

  #[must_use]
  /// Get the components imported by this [CompositeComponentConfiguration].
  pub fn components(&self) -> &HashMap<String, BoundComponent> {
    &self.import
  }

  #[must_use]
  /// Get an imported component by ID.
  pub fn component(&self, id: &str) -> Option<&BoundComponent> {
    self.import.iter().find(|(k, _)| *k == id).map(|(_, v)| v)
  }

  #[must_use]
  /// Get a map of [FlowOperation]s from the [CompositeComponentConfiguration]
  pub fn operations(&self) -> &HashMap<String, FlowOperation> {
    &self.operations
  }

  /// Get a [FlowOperation] by name.
  #[must_use]
  pub fn flow(&self, name: &str) -> Option<&FlowOperation> {
    self.operations.iter().find(|(n, _)| name == *n).map(|(_, v)| v)
  }
}
