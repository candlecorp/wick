use std::collections::HashMap;

use crate::config::common::flow_definition::FlowOperation;

#[derive(Debug, Default, Clone, derive_asset_container::AssetManager)]
#[asset(crate::config::AssetReference)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct CompositeComponentImplementation {
  #[asset(skip)]
  pub(crate) operations: HashMap<String, FlowOperation>,
}

impl CompositeComponentImplementation {
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

  /// Get the signature of the component as defined by the manifest.
  #[must_use]
  pub fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.values().cloned().map(Into::into).collect()
  }
}

impl From<FlowOperation> for wick_interface_types::OperationSignature {
  fn from(operation: FlowOperation) -> Self {
    Self {
      name: operation.name,
      inputs: operation.inputs,
      outputs: operation.outputs,
    }
  }
}
