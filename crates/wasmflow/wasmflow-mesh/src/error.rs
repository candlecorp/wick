use thiserror::Error;
use tokio::task::JoinError;
use tokio::time::error::Elapsed;

#[derive(Error, Debug)]
pub enum MeshError {
  #[error("Connection to mesh failed: {0}")]
  ConnectionFailed(std::io::Error),

  #[error("Could not create or retrieve stream: {0}")]
  CreateStream(std::io::Error),

  #[error("Could not create or open consumer: {0}")]
  CreateOrOpenConsumer(std::io::Error),

  #[error("Could not get stream info: {0}")]
  GetStreamInfo(std::io::Error),

  #[error("Error during shutdown: {0}")]
  ShutdownError(std::io::Error),

  #[error("Could not respond to message: {0}")]
  ResponseFail(std::io::Error),

  #[error("Could not retrieve mesh message for {0}: {1}")]
  RetrieveError(String, String),

  #[error("Could not deserialize RPC message: {0}")]
  MessageDeserialization(String),

  #[error("Could not serialize RPC message: {0}")]
  MessageSerialization(String),

  #[error("Could not acquire lock on consumer")]
  LockError,

  #[error("Failure publishing a message to nats: {0}")]
  PublishFail(String),

  #[error("Failure request a message to nats: {0}")]
  RequestFail(String),

  #[error("Failure subscribing to a subject: {0}")]
  SubscribeFail(String),

  #[error("Failed to send mesh response, channel closed")]
  ResponseUpstreamClosed,

  #[error("Failure while spawning a task to handle NATS command: {0}")]
  SpawnFail(String),

  #[error("Can not invoke non-component entities across the mesh")]
  InvalidEntity,

  #[error("Failed to query list of namespaces on mesh: {0}")]
  ListFail(String),

  #[error("To initialize a Mesh from the environment, {0} must be set")]
  NatsEnvVar(String),

  #[error("Timeout out waiting for result from mesh: {0}")]
  WaitTimeout(Elapsed),

  #[error("Invalid file path: {0}")]
  BadPath(String),
}

impl From<JoinError> for MeshError {
  fn from(e: JoinError) -> Self {
    MeshError::SpawnFail(e.to_string())
  }
}
