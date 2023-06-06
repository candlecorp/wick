#![allow(missing_docs)] // delete when we move away from the `property` crate.
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

  /// A list of operations to expose on this component.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) operations: Vec<SqlOperationDefinition>,
}

impl SqlComponentConfig {
  /// Get the signature of the component as defined by the manifest.
  #[must_use]
  pub fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.clone().into_iter().map(Into::into).collect()
  }
}

impl From<SqlOperationDefinition> for wick_interface_types::OperationSignature {
  fn from(operation: SqlOperationDefinition) -> Self {
    Self {
      name: operation.name,
      inputs: operation.inputs,
      outputs: operation.outputs,
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
  pub(crate) name: String,

  /// Types of the inputs to the operation.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) inputs: Vec<wick_interface_types::Field>,

  /// Types of the outputs to the operation.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) outputs: Vec<wick_interface_types::Field>,

  /// The configuration the operation needs.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) config: Vec<wick_interface_types::Field>,

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
