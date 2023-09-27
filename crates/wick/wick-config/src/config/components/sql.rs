#![allow(missing_docs)]
use std::borrow::Cow;

// delete when we move away from the `property` crate.
use wick_interface_types::{Field, OperationSignatures};

use super::{ComponentConfig, OperationConfig};
use crate::config::bindings::BoundIdentifier;
use crate::config::{self, ErrorBehavior};
use crate::utils::impl_from_for;

#[derive(
  Debug,
  Clone,
  derive_builder::Builder,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(config::AssetReference))]
#[builder(setter(into))]
/// A component made out of other components
pub struct SqlComponentConfig {
  /// The TcpPort reference to listen on for connections.
  #[asset(skip)]
  pub(crate) resource: BoundIdentifier,

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
  pub(crate) operations: Vec<SqlOperationDefinition>,
}

impl SqlComponentConfig {}

impl OperationSignatures for SqlComponentConfig {
  fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.clone().into_iter().map(Into::into).collect()
  }
}

impl_from_for!(SqlOperationDefinition, Query, SqlQueryOperationDefinition);
impl_from_for!(SqlOperationDefinition, Exec, SqlExecOperationDefinition);

impl From<SqlOperationDefinition> for wick_interface_types::OperationSignature {
  fn from(value: SqlOperationDefinition) -> Self {
    match value {
      SqlOperationDefinition::Query(v) => v.into(),
      SqlOperationDefinition::Exec(v) => v.into(),
    }
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

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SqlOperationDefinition {
  Query(SqlQueryOperationDefinition),
  Exec(SqlExecOperationDefinition),
}

#[derive(Debug, Clone, Copy)]
pub enum SqlOperationKind {
  Query,
  Exec,
}

impl std::fmt::Display for SqlOperationKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SqlOperationKind::Query => write!(f, "sql query operation"),
      SqlOperationKind::Exec => write!(f, "sql exec operation"),
    }
  }
}

impl SqlOperationDefinition {
  #[must_use]
  pub const fn on_error(&self) -> ErrorBehavior {
    match self {
      SqlOperationDefinition::Query(v) => v.on_error,
      SqlOperationDefinition::Exec(v) => v.on_error,
    }
  }

  #[must_use]
  pub fn arguments(&self) -> &[String] {
    match self {
      SqlOperationDefinition::Query(v) => &v.arguments,
      SqlOperationDefinition::Exec(v) => &v.arguments,
    }
  }

  #[must_use]
  pub fn query(&self) -> &str {
    match self {
      SqlOperationDefinition::Query(v) => &v.query,
      SqlOperationDefinition::Exec(v) => &v.exec,
    }
  }

  #[must_use]
  pub const fn kind(&self) -> SqlOperationKind {
    match self {
      SqlOperationDefinition::Query(_) => SqlOperationKind::Query,
      SqlOperationDefinition::Exec(_) => SqlOperationKind::Exec,
    }
  }
}

impl OperationConfig for SqlOperationDefinition {
  fn name(&self) -> &str {
    match self {
      SqlOperationDefinition::Query(v) => &v.name,
      SqlOperationDefinition::Exec(v) => &v.name,
    }
  }

  fn inputs(&self) -> Cow<Vec<Field>> {
    match self {
      SqlOperationDefinition::Query(v) => v.inputs(),
      SqlOperationDefinition::Exec(v) => v.inputs(),
    }
  }

  fn outputs(&self) -> Cow<Vec<Field>> {
    match self {
      SqlOperationDefinition::Query(v) => v.outputs(),
      SqlOperationDefinition::Exec(v) => v.outputs(),
    }
  }
}

impl OperationConfig for SqlQueryOperationDefinition {
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

impl From<SqlQueryOperationDefinition> for wick_interface_types::OperationSignature {
  fn from(operation: SqlQueryOperationDefinition) -> Self {
    // TODO: Properly use configured outputs here.
    // Forcing SQL components to have a single object output called "output" is a temporary
    // limitation
    let outputs = vec![Field::new("output", wick_interface_types::Type::Object)];

    Self::new(operation.name, operation.inputs, outputs, operation.config)
  }
}

impl From<SqlExecOperationDefinition> for wick_interface_types::OperationSignature {
  fn from(operation: SqlExecOperationDefinition) -> Self {
    let outputs = vec![Field::new("output", wick_interface_types::Type::U32)];

    Self::new(operation.name, operation.inputs, outputs, operation.config)
  }
}
#[derive(
  Debug,
  Clone,
  derive_builder::Builder,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(config::AssetReference))]
#[builder(setter(into))]
/// An operation whose implementation is a SQL query to execute on a database.
pub struct SqlQueryOperationDefinition {
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
  Debug,
  Clone,
  derive_builder::Builder,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
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
