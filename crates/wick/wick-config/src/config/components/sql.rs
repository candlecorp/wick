#[derive(Debug, Clone, PartialEq)]
/// A component made out of other components
pub struct SqlComponentConfig {
  /// The TcpPort reference to listen on for connections.
  pub resource: String,

  /// The kind of database to connect to.
  pub vendor: DatabaseKind,

  /// The username to use when connecting to the postgres database.
  pub user: String,

  /// The password to use when connecting to the postgres database.
  pub password: String,

  /// The database to connect to.
  pub database: String,

  /// Whether or not to use TLS.
  pub tls: bool,

  /// A list of operations to expose on this component.
  pub operations: Vec<SqlOperationDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SqlOperationDefinition {
  /// The name of the operation.
  pub name: String,

  /// Types of the inputs to the operation.
  pub inputs: Vec<wick_interface_types::Field>,

  /// Types of the outputs to the operation.
  pub outputs: Vec<wick_interface_types::Field>,

  /// The query to execute.
  pub query: String,

  /// The arguments to the query, defined as a list of input names.
  pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatabaseKind {
  MsSql = 0,
  Postgres = 1,
  Mysql = 2,
  Sqlite = 3,
}
