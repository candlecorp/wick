#[derive(Debug, Clone, PartialEq)]
/// A component made out of other components
pub struct PostgresComponent {
  /// The TcpPort reference to listen on for connections.
  pub resource: String,

  /// The username to use when connecting to the postgres database.
  pub user: String,

  /// The password to use when connecting to the postgres database.
  pub password: String,

  /// The database to connect to.
  pub database: String,

  /// Whether or not to use TLS.
  pub tls: bool,

  /// A list of operations to expose on this component.
  pub operations: Vec<PostgresOperationDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostgresOperationDefinition {
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
