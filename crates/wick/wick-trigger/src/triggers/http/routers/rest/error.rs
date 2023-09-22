#[derive(Debug, thiserror::Error)]
pub(crate) enum RestError {
  #[error("Missing configuration necessary to generate OpenApi spec: {0}")]
  MissingConfig(String),

  #[error("Rest router referenced type {0} but {0} was not found")]
  TypeNotFound(String),
}
