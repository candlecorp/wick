use flow_component::ComponentError;
use wick_config::error::ManifestError;
use wick_packet::TypeWrapper;

#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Error {
  #[error("Invalid output for operations {}. At this time postgres operations can have at most one output named 'output' of type 'object'", .0.join(", "))]
  InvalidOutput(Vec<String>),

  #[error("Failed to fetch result of query: {0}")]
  Fetch(String),

  #[error("Failed to fetch result of exec: {0}")]
  Exec(String),

  #[error("Unknown database scheme '{0}'")]
  InvalidScheme(String),

  #[error(
    "To use in-memory SQLite databases, use the URL 'sqlite://memory'; to use a SQLite DB file, use a 'file://' URL"
  )]
  SqliteScheme,

  #[error("Failed to prepare arguments: {0}")]
  Prepare(String),

  #[error("Failed to connect to MsSqlServer: {0}")]
  MssqlConnect(String),

  #[error("Failed to connect to Postgres Server: {0}")]
  PostgresConnect(String),

  #[error("Failed to open to Sqlite DB: {0}")]
  SqliteConnect(String),

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

  #[error("SQL Query failed, check log for details")]
  QueryFailed,

  #[error("SQL error reported within stream: {0}")]
  ErrorInStream(String),

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

  #[error("Could not encode wick type {} with value '{}' into the DB's type for {1}. Try a different value, type, or coersion within the SQL query.",.0.type_signature(),.0.inner())]
  SqlServerEncodingFault(TypeWrapper, ConversionError),

  #[error(transparent)]
  ComponentError(wick_packet::Error),

  #[error("Database connection not initialized")]
  Uninitialized,

  #[error(transparent)]
  Configuration(#[from] ManifestError),

  #[error("Resource valid but its value could not be retrieved")]
  InvalidResourceConfig,

  #[error("Got a row with no data")]
  NoRow,
}

impl From<Error> for ComponentError {
  fn from(value: Error) -> Self {
    ComponentError::new(value)
  }
}

#[derive(thiserror::Error, Debug, Copy, Clone)]
pub enum ConversionError {
  #[error("i8")]
  I8,
  #[error("i16")]
  I16,
  #[error("i32")]
  I32,
  #[error("i64")]
  I64,
  #[error("u8")]
  U8,
  #[error("u16")]
  U16,
  #[error("u32")]
  U32,
  #[error("u64")]
  U64,
  #[error("f32")]
  F32,
  #[error("f64")]
  F64,
  #[error("bool")]
  Bool,
  #[error("string")]
  String,
  #[error("datetime")]
  Datetime,
  #[error("bytes")]
  Bytes,
  #[error("named")]
  Named,
  #[error("list")]
  List,
  #[error("optional")]
  Optional,
  #[error("map")]
  Map,
  #[error("link")]
  Link,
  #[error("object")]
  Object,
  #[error("anonymous struct")]
  AnonymousStruct,
}
