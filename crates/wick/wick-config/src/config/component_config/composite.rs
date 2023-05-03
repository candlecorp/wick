use std::collections::HashMap;

use flow_expression_parser::ast::{self};
use wick_interface_types::Field;
use wick_packet::OperationConfig;

use crate::config::{self};

#[derive(Debug, Default, Clone, derive_asset_container::AssetManager)]
#[asset(asset(crate::config::AssetReference))]
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

#[derive(Debug, Clone, Default)]
/// The SchematicDefinition struct is a normalized representation of a Wick [SchematicManifest].
/// It handles the job of translating manifest versions into a consistent data structure.
#[must_use]
pub struct FlowOperation {
  /// The name of the schematic.
  pub name: String,

  /// A list of the input types for the operation.
  pub inputs: Vec<Field>,

  /// A list of the input types for the operation.
  pub outputs: Vec<Field>,

  /// Any configuration required for the component to operate.
  pub config: Vec<Field>,

  /// A mapping of instance names to the components they refer to.
  pub instances: HashMap<String, InstanceReference>,

  /// A list of connections from and to ports on instances defined in the instance map.
  pub expressions: Vec<ast::FlowExpression>,

  /// Additional flows scoped to this operation.
  pub flows: Vec<FlowOperation>,

  /// A list of component IDs to expose to this schematic.
  pub components: Vec<String>,
}

impl FlowOperation {
  /// Get the name as an owned [String].
  #[must_use]
  pub fn get_name(&self) -> String {
    self.name.clone()
  }
  /// Get a [ComponentDefinition] by instance name.
  #[must_use]
  pub fn get_component(&self, instance: &str) -> Option<InstanceReference> {
    self.instances.get(instance).cloned()
  }

  /// Get a reference to the [ComponentDefinition] map.
  #[must_use]
  pub fn instances(&self) -> &HashMap<String, InstanceReference> {
    &self.instances
  }
}

impl From<FlowOperation> for config::OperationSignature {
  fn from(value: FlowOperation) -> Self {
    Self {
      name: value.name,
      inputs: value.inputs,
      outputs: value.outputs,
      config: value.config,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
/// A definition of a component used to reference a component registered under a collection.
/// Note: [InstanceReference] include embed the concept of a namespace so two identical.
/// components registered on different namespaces will not be equal.
pub struct InstanceReference {
  /// The operation's name.
  pub name: String,
  /// The id of the component.
  pub component_id: String,
  /// Data associated with the component instance.
  pub data: Option<OperationConfig>,
}

impl InstanceReference {
  /// Returns the fully qualified ID for the component, i.e. namespace::name.
  #[must_use]
  pub fn id(&self) -> String {
    format!("{}::{}", self.component_id, self.name)
  }
}

impl std::fmt::Display for InstanceReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.id())
  }
}
