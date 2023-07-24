#![allow(missing_docs)]
use std::borrow::Cow;

// delete when we move away from the `property` crate.
use wick_interface_types::{Field, OperationSignatures};

use super::{ComponentConfig, OperationConfig};
use crate::config::{self, ErrorBehavior};

#[derive(
  Debug, Clone, Builder, PartialEq, derive_asset_container::AssetManager, property::Property, serde::Serialize,
)]
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
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) config: Vec<Field>,

  /// A list of operations to expose on this component.
  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<SqlOperationKind>,
}

impl SqlComponentConfig {}

impl OperationSignatures for SqlComponentConfig {
  fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.clone().into_iter().map(Into::into).collect()
  }
}

impl From<SqlOperationKind> for wick_interface_types::OperationSignature {
  fn from(value: SqlOperationKind) -> Self {
    match value {
      SqlOperationKind::Query(v) => v.into(),
      SqlOperationKind::Exec(v) => v.into(),
    }
  }
}

impl ComponentConfig for SqlComponentConfig {
  type Operation = SqlOperationKind;

  fn operations(&self) -> &[Self::Operation] {
    &self.operations
  }

  fn operations_mut(&mut self) -> &mut Vec<Self::Operation> {
    &mut self.operations
  }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SqlOperationKind {
  Query(SqlOperationDefinition),
  Exec(SqlExecOperationDefinition),
}

impl SqlOperationKind {
  #[must_use]
  pub fn on_error(&self) -> ErrorBehavior {
    match self {
      SqlOperationKind::Query(v) => v.on_error,
      SqlOperationKind::Exec(v) => v.on_error,
    }
  }

  #[must_use]
  pub fn arguments(&self) -> &[String] {
    match self {
      SqlOperationKind::Query(v) => &v.arguments,
      SqlOperationKind::Exec(v) => &v.arguments,
    }
  }

  #[must_use]
  pub fn query(&self) -> &str {
    match self {
      SqlOperationKind::Query(v) => &v.query,
      SqlOperationKind::Exec(v) => &v.exec,
    }
  }
}

impl OperationConfig for SqlOperationKind {
  fn name(&self) -> &str {
    match self {
      SqlOperationKind::Query(v) => &v.name,
      SqlOperationKind::Exec(v) => &v.name,
    }
  }

  fn inputs(&self) -> Cow<Vec<Field>> {
    match self {
      SqlOperationKind::Query(v) => v.inputs(),
      SqlOperationKind::Exec(v) => v.inputs(),
    }
  }

  fn outputs(&self) -> Cow<Vec<Field>> {
    match self {
      SqlOperationKind::Query(v) => v.outputs(),
      SqlOperationKind::Exec(v) => v.outputs(),
    }
  }
}

impl OperationConfig for SqlOperationDefinition {
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

impl OperationConfig for SqlExecOperationDefinition {
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

impl From<SqlExecOperationDefinition> for wick_interface_types::OperationSignature {
  fn from(operation: SqlExecOperationDefinition) -> Self {
    let outputs = vec![Field::new("output", wick_interface_types::Type::U32)];

    Self {
      name: operation.name,
      config: operation.config,
      inputs: operation.inputs,
      outputs,
    }
  }
}
#[derive(
  Debug, Clone, Builder, PartialEq, derive_asset_container::AssetManager, property::Property, serde::Serialize,
)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
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
  #[property(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<Field>,

  /// Types of the outputs to the operation.
  #[asset(skip)]
  #[property(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<Field>,

  /// The configuration the operation needs.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) config: Vec<Field>,

  /// The query to execute.
  #[asset(skip)]
  pub(crate) query: String,

  /// The arguments to the query, defined as a list of input names.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) arguments: Vec<String>,

  /// The query to execute.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) on_error: ErrorBehavior,
}

#[derive(
  Debug, Clone, Builder, PartialEq, derive_asset_container::AssetManager, property::Property, serde::Serialize,
)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(config::AssetReference))]
#[builder(setter(into))]
/// An operation whose implementation is a SQL query to execute on a database and returns the number of rows affected or failure.
pub struct SqlExecOperationDefinition {
  /// The name of the operation.
  #[asset(skip)]
  #[property(skip)]
  pub(crate) name: String,

  /// Types of the inputs to the operation.
  #[asset(skip)]
  #[property(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<Field>,

  /// Types of the outputs to the operation.
  #[asset(skip)]
  #[property(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<Field>,

  /// The configuration the operation needs.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) config: Vec<Field>,

  /// The query to execute.
  #[asset(skip)]
  pub(crate) exec: String,

  /// The arguments to the query, defined as a list of input names.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) arguments: Vec<String>,

  /// The query to execute.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) on_error: ErrorBehavior,
}
