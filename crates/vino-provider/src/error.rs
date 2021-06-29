use thiserror::Error;

type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

#[derive(Error, Debug)]
pub enum ProviderError {
  #[error("Error initializing provider")]
  InitError,
  #[error("Provider is not initialized")]
  Uninitialized,
  #[error("Provider is already started")]
  AlreadyStarted,
  #[error("Component '{0}' not found on this provider")]
  ComponentNotFound(String),
  #[error("Invalid state for component '{0}'")]
  JobError(String),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error("Error serializing payload")]
  SerializationError(BoxedSyncSendError),
  #[error("Error deserializing job input {0}")]
  InputDeserializationError(BoxedSyncSendError),
  #[error("Error deserializing job payload {0}")]
  PayloadDeserializationError(BoxedSyncSendError),
  #[error("General error : {0}")]
  Other(String),
  #[error(transparent)]
  OtherUpstreamError(#[from] Box<dyn std::error::Error + Send + Sync>),
}
