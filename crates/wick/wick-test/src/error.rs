use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TestError {
  #[error("Could not read file : {0}")]
  ReadFailed(String),
  #[error("Could not parse contents as YAML : {0}")]
  ParseFailed(String),
  #[error("Invocation failed: {0}")]
  InvocationFailed(String),
  #[error("Deserialization failed: {0}")]
  ConversionFailed(String),
}
