use wick_trigger::resources::ResourceKind;

#[derive(Debug, Clone, Copy)]
#[allow(clippy::exhaustive_enums)]
pub enum ErrorContext {
  Http,
  Time,
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
  #[error("expected a component reference but found an unimported definition, this is a bug")]
  InvalidReference,

  #[error("could not find resource by ID '{0}'")]
  ResourceNotFound(String),

  #[error("expected {0} resource, got a {1}")]
  InvalidResourceType(ResourceKind, ResourceKind),

  #[error("{0}")]
  ShutdownFailed(String),

  #[error(transparent)]
  Runtime(Box<wick_runtime::error::RuntimeError>),
}

impl From<wick_runtime::error::RuntimeError> for Error {
  fn from(value: wick_runtime::error::RuntimeError) -> Self {
    Error::Runtime(Box::new(value))
  }
}
