use flow_component::ComponentError;
use wick_config::error::ManifestError;

#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
  #[error("Invalid output for operations {}. At this time postgres operations can have at most one output named 'output' of type 'object'", .0.join(", "))]
  InvalidOutput(Vec<String>),

  #[error("Failed to fetch result of query: {0}")]
  Fetch(String),

  #[error("Unknown database scheme '{0}'")]
  InvalidScheme(String),

  #[error("Failed to prepare arguments: {0}")]
  Prepare(String),

  #[error("Failed to connect to MsSqlServer: {0}")]
  MssqlConnect(String),

  #[error("Failed to connect to Postgres Server: {0}")]
  PostgresConnect(String),

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

  #[error("Missing positional argument '{0}'")]
  MissingArgument(String),

  #[error("Missing input")]
  MissingInput,

  #[error("Operation '{0}' not found on this component")]
  MissingOperation(String),

  #[error("Could not find a value for input '{0}' to bind to a positional argument")]
  MissingPacket(String),

  #[error(transparent)]
  ComponentError(wick_packet::Error),

  #[error("Could not resolve resource: {0}")]
  ResourceNotFound(String),

  #[error("Database connection not initialized")]
  Uninitialized,

  #[error("Invalid resource passed to component: {0}")]
  InvalidResource(ManifestError),

  #[error("Resource valid but its value could not be retrieved")]
  InvalidResourceConfig,
}

impl From<Error> for ComponentError {
  fn from(value: Error) -> Self {
    ComponentError::new(value)
  }
}
