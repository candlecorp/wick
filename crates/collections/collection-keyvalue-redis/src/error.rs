use thiserror::Error;
type BoxedError = Box<dyn std::error::Error + Sync + Send>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Could not coerce the cursor ('{0}') to a u64 value.")]
  CursorConversion(String),

  #[error("Error during initialization: {0}")]
  Init(String),

  #[error("Can't get lock on context")]
  ContextLock,

  #[error("No connection found for '{0}'")]
  ConnectionNotFound(String),

  #[error("Redis command failed: {0}")]
  RedisError(String),

  #[error("Deserialization error {0}")]
  RpcMessageError(&'static str),

  #[error("Client is shutting down, streams are closing")]
  ShuttingDown,

  #[error(transparent)]
  RpcError(#[from] wick_rpc::Error),

  #[error(transparent)]
  CliError(#[from] wick_component_cli::Error),

  #[error(transparent)]
  IoError(#[from] std::io::Error),

  #[error(transparent)]
  UpstreamError(#[from] BoxedError),
}

pub(crate) enum Exception {
  KeyNotFound(String),
}

impl From<Exception> for String {
  fn from(e: Exception) -> Self {
    e.to_string()
  }
}

impl std::fmt::Display for Exception {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Exception::KeyNotFound(v) => write!(f, "Key '{}' not found", v),
      // Exception::Other(v) => write!(f, "{}", v),
    }
  }
}
