use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum LatticeError {
  #[error(transparent)]
  ConnectionError(#[from] std::io::Error),

  #[error("Could not retrieve lattice message for {0}: {1}")]
  RetrieveError(String, String),

  #[error("Could not deserialize RPC message: {0}")]
  MessageDeserialization(String),

  #[error("Could not serialize RPC message: {0}")]
  MessageSerialization(String),

  #[error("Could not acquire lock on consumer")]
  LockError,

  #[error("Failure publishing a message to nats: {0}")]
  PublishFail(String),

  #[error("Failure subscribing to a subject: {0}")]
  SubscribeFail(String),

  #[error("Failure while spawning a task to handle NATS command: {0}")]
  SpawnFail(String),

  #[error("Can not invoke non-component entities across the lattice.")]
  InvalidEntity,

  #[error("Failed to query list of namespaces on lattice: {0}")]
  ListFail(String),

  #[error("To initialize a Lattice from the environment, {0} must be set.")]
  NatsEnvVar(String),

  #[error("Timeout out waiting for result from lattice")]
  WaitTimeout,

  #[error("Invalid file path: {0}")]
  BadPath(String),
}

impl From<JoinError> for LatticeError {
  fn from(e: JoinError) -> Self {
    LatticeError::SpawnFail(e.to_string())
  }
}
