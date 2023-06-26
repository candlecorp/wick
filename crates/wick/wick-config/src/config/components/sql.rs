#![allow(missing_docs)] // delete when we move away from the `property` crate.
use wick_interface_types::{Field, OperationSignatures};

use super::{ComponentConfig, OperationConfig};
use crate::config::{self, ErrorBehavior};

#[derive(Debug, Clone, Builder, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(config::AssetReference))]
#[builder(setter(into))]
/// A component made out of other components
pub struct SqlComponentConfig {
  /// The TcpPort reference to listen on for connections.
  #[asset(skip)]
  pub(crate) resource: String,

  /// Whether or not to use TLS.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) tls: bool,

  /// The configuration for the component.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) config: Vec<Field>,

  /// A list of operations to expose on this component.
  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  pub(crate) operations: Vec<SqlOperationDefinition>,
}

impl SqlComponentConfig {}

impl OperationSignatures for SqlComponentConfig {
  fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.clone().into_iter().map(Into::into).collect()
  }
}

impl ComponentConfig for SqlComponentConfig {
  type Operation = SqlOperationDefinition;

  fn operations(&self) -> &[Self::Operation] {
    &self.operations
  }

  fn operations_mut(&mut self) -> &mut Vec<Self::Operation> {
    &mut self.operations
  }
}

impl OperationConfig for SqlOperationDefinition {
  fn name(&self) -> &str {
    &self.name
  }
}

impl From<SqlOperationDefinition> for wick_interface_types::OperationSignature {
  fn from(operation: SqlOperationDefinition) -> Self {
    // TODO: Properly use configured outputs here.
    // Forcing SQL components to have a single object output called "output" is a temporary
    // limitation
    let outputs = vec![Field::new("output", wick_interface_types::Type::Object)];

    Self {
      name: operation.name,
      config: operation.config,
      inputs: operation.inputs,
      outputs,
    }
  }
}

#[derive(Debug, Clone, Builder, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]
#[builder(setter(into))]
/// An operation whose implementation is a SQL query to execute on a database.
pub struct SqlOperationDefinition {
  /// The name of the operation.
  #[asset(skip)]
  #[property(skip)]
  pub(crate) name: String,

  /// Types of the inputs to the operation.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) inputs: Vec<Field>,

  /// Types of the outputs to the operation.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) outputs: Vec<Field>,

  /// The configuration the operation needs.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) config: Vec<Field>,

  /// The query to execute.
  #[asset(skip)]
  pub(crate) query: String,

  /// The arguments to the query, defined as a list of input names.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) arguments: Vec<String>,

  /// The query to execute.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) on_error: ErrorBehavior,
}
