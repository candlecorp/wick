use std::borrow::Cow;
use std::collections::HashMap;

use flow_expression_parser::ast::{self};
use wick_interface_types::{Field, OperationSignatures};

use crate::config::components::{ComponentConfig, OperationConfig};
use crate::config::{self, ExecutionSettings, LiquidJsonConfig};

#[derive(Debug, Default, Clone, derive_asset_container::AssetManager, Builder, property::Property)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[builder(setter(into))]
#[asset(asset(crate::config::AssetReference))]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct CompositeComponentImplementation {
  /// The operations defined by the component.
  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  pub(crate) operations: Vec<FlowOperation>,

  /// The configuration for the component.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) config: Vec<Field>,

  /// A component id to inherit operations from.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) inherit: Option<String>,
}

impl CompositeComponentImplementation {
  /// Get a [FlowOperation] by name.
  #[must_use]
  pub fn flow(&self, name: &str) -> Option<&FlowOperation> {
    self.operations.iter().find(|n| n.name() == name)
  }
}

impl OperationSignatures for CompositeComponentImplementation {
  fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.iter().cloned().map(Into::into).collect()
  }
}

impl ComponentConfig for CompositeComponentImplementation {
  type Operation = FlowOperation;

  fn operations(&self) -> &[Self::Operation] {
    &self.operations
  }

  fn operations_mut(&mut self) -> &mut Vec<Self::Operation> {
    &mut self.operations
  }
}

impl OperationConfig for FlowOperation {
  fn name(&self) -> &str {
    &self.name
  }

  fn inputs(&self) -> Cow<Vec<Field>> {
    Cow::Borrowed(&self.inputs)
  }

  fn outputs(&self) -> Cow<Vec<Field>> {
    Cow::Borrowed(&self.outputs)
  }
}

impl From<FlowOperation> for wick_interface_types::OperationSignature {
  fn from(operation: FlowOperation) -> Self {
    Self {
      name: operation.name,
      config: operation.config,
      inputs: operation.inputs,
      outputs: operation.outputs,
    }
  }
}

#[derive(Debug, Clone, Builder, Default, property::Property)]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]
#[builder(setter(into))]
/// A FlowOperation is an operation definition whose implementation is defined by
/// connecting other components together in a flow or set of pipelines.
#[must_use]
pub struct FlowOperation {
  /// The name of the schematic.
  #[property(skip)]
  pub(crate) name: String,

  /// A list of the input types for the operation.
  #[builder(default)]
  #[property(skip)]
  pub(crate) inputs: Vec<Field>,

  /// A list of the input types for the operation.
  #[builder(default)]
  #[property(skip)]
  pub(crate) outputs: Vec<Field>,

  /// Any configuration required for the component to operate.
  #[builder(default)]
  pub(crate) config: Vec<Field>,

  /// A mapping of instance names to the components they refer to.
  #[builder(default)]
  pub(crate) instances: HashMap<String, InstanceReference>,

  /// A list of connections from and to ports on instances defined in the instance map.
  #[builder(default)]
  pub(crate) expressions: Vec<ast::FlowExpression>,

  /// Additional flows scoped to this operation.
  #[builder(default)]
  pub(crate) flows: Vec<FlowOperation>,
}

impl From<FlowOperation> for config::OperationDefinition {
  fn from(value: FlowOperation) -> Self {
    Self {
      name: value.name,
      inputs: value.inputs,
      outputs: value.outputs,
      config: value.config,
    }
  }
}

#[derive(Debug, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// A definition of a component used to reference a component registered under a collection.
/// Note: [InstanceReference] include embed the concept of a namespace so two identical.
/// components registered on different namespaces will not be equal.
pub struct InstanceReference {
  /// The operation's name.
  pub(crate) name: String,
  /// The id of the component.
  pub(crate) component_id: String,
  /// Data associated with the component instance.
  pub(crate) data: Option<LiquidJsonConfig>,
  /// Per-operation settings that override global execution settings.
  pub(crate) settings: Option<ExecutionSettings>,
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
