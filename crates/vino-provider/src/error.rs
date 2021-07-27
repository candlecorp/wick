use thiserror::Error;
use vino_rpc::error::RpcError;

#[derive(Error, Debug)]
/// Vino Provider's error type
pub enum ProviderError {
  /// Error returned when a component can not be found.
  #[error("Component '{0}' not found on this provider")]
  ComponentNotFound(String),
  /// IO error
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  /// Unspecified upstream error
  #[error(transparent)]
  OtherUpstreamError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl From<ProviderError> for Box<RpcError> {
  fn from(e: ProviderError) -> Self {
    Box::new(RpcError::ProviderError(e.to_string()))
  }
}

#[derive(Error, Debug)]
#[must_use]
/// The error type that components can return on failures.
pub struct ProviderComponentError {
  msg: String,
}

impl std::fmt::Display for ProviderComponentError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.msg)
  }
}

impl ProviderComponentError {
  /// Constructor for [ProviderComponentError]
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
