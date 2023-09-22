use crate::resources::ResourceKind;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub struct Error {
  pub context: Option<ErrorContext>,
  #[source]
  pub kind: ErrorKind,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.kind.fmt(f)
  }
}

impl Error {
  pub(crate) const fn new(kind: ErrorKind) -> Self {
    Self { kind, context: None }
  }
  pub(crate) const fn new_context(context: ErrorContext, kind: ErrorKind) -> Self {
    Self {
      kind,
      context: Some(context),
    }
  }
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::exhaustive_enums)]
pub enum ErrorContext {
  Http,
  Time,
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ErrorKind {
  #[error("expected a component reference but found an unimported definition, this is a bug")]
  InvalidReference,

  #[error("could not find resource by ID '{0}'")]
  ResourceNotFound(String),

  #[error("expected {0} resource, got a {1}")]
  InvalidResourceType(ResourceKind, ResourceKind),

  #[error("{0}")]
  ShutdownFailed(String),

  #[error(transparent)]
  Time(Box<crate::triggers::time::error::TimeError>),

  #[error(transparent)]
  Http(Box<crate::triggers::http::error::HttpError>),

  #[error(transparent)]
  Runtime(Box<wick_runtime::error::RuntimeError>),
}

impl From<wick_runtime::error::RuntimeError> for Error {
  fn from(value: wick_runtime::error::RuntimeError) -> Self {
    Self::new(ErrorKind::Runtime(Box::new(value)))
  }
}
