use wick_config::config::BoundIdentifier;

use crate::resources::ResourceKind;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub struct Error {
  pub context: Option<String>,
  #[source]
  pub kind: ErrorKind,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.kind.fmt(f)
  }
}

impl Error {
  #[must_use]
  pub const fn new(kind: ErrorKind) -> Self {
    Self { kind, context: None }
  }
  pub fn new_context<T: Into<String>>(context: T, kind: ErrorKind) -> Self {
    Self {
      kind,
      context: Some(context.into()),
    }
  }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ErrorKind {
  #[error("error during trigger startup: {0}")]
  Startup(String),

  #[error("error during trigger shutdown: {0}")]
  Shutdown(String),

  #[error("internal error in trigger: {0}")]
  InternalError(String),

  #[error("error in trigger: {0}")]
  Trigger(Box<dyn std::error::Error + Send + Sync>),

  #[error("could not find resource by ID '{0}'")]
  ResourceNotFound(BoundIdentifier),

  #[error("expected {0} resource, got a {1}")]
  InvalidResourceType(ResourceKind, ResourceKind),

  #[error(transparent)]
  Runtime(Box<wick_runtime::error::RuntimeError>),
}

impl From<wick_runtime::error::RuntimeError> for Error {
  fn from(value: wick_runtime::error::RuntimeError) -> Self {
    Self::new(ErrorKind::Runtime(Box::new(value)))
  }
}
