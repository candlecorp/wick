use flow_component::ComponentError;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
  #[error("Invalid float")]
  InvalidFloat,
  #[error("Cannot convert pg-cell \"{0}\" of type \"{1}\" to a JSONValue.")]
  Conversion(String, String),
  #[error("Invalid output for operations {}. At this time postgres operations can have at most one output named 'output' of type 'object'", .0.join(", "))]
  InvalidOutput(Vec<String>),
  #[error("Failed to fetch result of query: {0}")]
  Fetch(String),

  #[error("Unknown database scheme '{0}'")]
  InvalidScheme(String),

  #[error("Failed to connect to MsSqlServer: {0}")]
  MssqlConnect(String),

  #[error("Failed to connect to Postgres Server: {0}")]
  PostgresConnect(String),

  #[error("Bad configuration: {0}")]
  Validation(String),

  #[error("Failed to prepare arguments: {0}")]
  Prepare(String),

  #[error("Failed to connect: {0}")]
  Connection(String),

  #[error("{0}")]
  Pool(String),

  #[error("Failed to get connection from pool: {0}")]
  PoolConnection(String),

  #[error("Failed to start DB transaction")]
  TxStart,

  #[error("Failed to commit DB transaction")]
  TxCommit,

  #[error("Failed to rollback DB transaction")]
  TxRollback,

  #[error("Operation failed: {0}")]
  OperationFailed(String),

  #[error("Query failed: {0}")]
  Failed(String),

  #[error("Missing input")]
  MissingInput,

  #[error(transparent)]
  ComponentError(wick_packet::Error),
}

impl From<Error> for ComponentError {
  fn from(value: Error) -> Self {
    ComponentError::new(value)
  }
}
