use crate::config;

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(config::AssetReference))]
/// A component made out of other components
pub struct SqlComponentConfig {
  /// The TcpPort reference to listen on for connections.
  #[asset(skip)]
  pub resource: String,

  /// Whether or not to use TLS.
  #[asset(skip)]
  pub tls: bool,

  /// A list of operations to expose on this component.
  #[asset(skip)]
  pub operations: Vec<SqlOperationDefinition>,
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

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(config::AssetReference))]

pub struct SqlOperationDefinition {
  /// The name of the operation.
  #[asset(skip)]
  pub name: String,

  /// Types of the inputs to the operation.
  #[asset(skip)]
  pub inputs: Vec<wick_interface_types::Field>,

  /// Types of the outputs to the operation.
  #[asset(skip)]
  pub outputs: Vec<wick_interface_types::Field>,

  /// The query to execute.
  #[asset(skip)]
  pub query: String,

  /// The arguments to the query, defined as a list of input names.
  #[asset(skip)]
  pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(config::AssetReference))]
pub enum DatabaseKind {
  MsSql = 0,
  Postgres = 1,
  Mysql = 2,
  Sqlite = 3,
}
