use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use vino_component::InvocationPacket;

#[derive(Error, Debug)]
/// Vino Provider's error type
pub enum Error {
  /// Error returned when a component can not be found.
  #[error("Component '{0}' not found on this provider")]
  ComponentNotFound(String),

  /// Send error
  #[error(transparent)]
  IOError(#[from] std::io::Error),

  /// Error sending output to channel
  #[error("Error sending output to channel")]
  SendError,

  /// Unspecified upstream error
  #[error(transparent)]
  OtherUpstreamError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Error, Debug)]
#[must_use]
/// The error type that components can return on failures.
pub struct NativeComponentError {
  msg: String,
}

impl std::fmt::Display for NativeComponentError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.msg)
  }
}

impl NativeComponentError {
  /// Constructor for [ProviderComponentError]
  pub fn new<T: AsRef<str>>(msg: T) -> Self {
    Self {
      msg: msg.as_ref().to_owned(),
    }
  }
}

impl From<&'static str> for NativeComponentError {
  fn from(e: &'static str) -> Self {
    NativeComponentError::new(e.to_owned())
  }
}

impl From<String> for NativeComponentError {
  fn from(e: String) -> Self {
    NativeComponentError::new(e)
  }
}

impl From<SendError<InvocationPacket>> for Error {
  fn from(_: SendError<InvocationPacket>) -> Self {
    Self::SendError
  }
}

impl From<Error> for NativeComponentError {
  fn from(e: Error) -> Self {
    Self::new(e.to_string())
  }
}
