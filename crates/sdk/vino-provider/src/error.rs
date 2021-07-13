use thiserror::Error;
use vino_rpc::error::RpcError;

type BoxedSyncSendError = Box<dyn std::error::Error + Sync + Send>;

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

impl From<ProviderError> for Box<RpcError> {
  fn from(e: ProviderError) -> Self {
    Box::new(RpcError::ProviderError(e.to_string()))
  }
}

#[derive(Error, Debug)]
pub struct ProviderComponentError {
  msg: String,
}

impl std::fmt::Display for ProviderComponentError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.msg)
  }
}

impl ProviderComponentError {
  pub fn new<T: AsRef<str>>(msg: T) -> Self {
    Self {
      msg: msg.as_ref().to_owned(),
    }
  }
}

impl From<Box<ProviderComponentError>> for Box<RpcError> {
  fn from(e: Box<ProviderComponentError>) -> Self {
    Box::new(RpcError::ComponentError(e.to_string()))
  }
}
