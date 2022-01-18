use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpError {
  #[error("Could not serialize response: {0}")]
  Serialization(String),
  #[error("Invocation failed: {0}")]
  InvocationFailed(String),
}
