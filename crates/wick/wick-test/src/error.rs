use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TestError {
  #[error("Could not read file : {0}")]
  ReadFailed(String),
  #[error("Could not parse contents as YAML : {0}")]
  ParseFailed(String),
  #[error("Invocation failed: {0}")]
  InvocationFailed(String),
  #[error("Invocation timed out: {0}")]
  InvocationTimeout(String),
  #[error("Serialization failed: {0}")]
  Serialization(String),
  #[error("Deserialization failed: {0}")]
  Deserialization(String),
  #[error("Could not render configuration: {0}")]
  Configuration(String),
  #[error("Could not create component instance to test: {0}")]
  Factory(String),
  #[error("Could not find operation {0} on this component")]
  OpNotFound(String),
  #[error(transparent)]
  ConfigUnsatisfied(wick_packet::Error),
}
