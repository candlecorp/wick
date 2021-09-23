use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
  #[error("Could not read file : {0}")]
  ReadFailed(String),

  #[error("Could not parse contents as YAML : {0}")]
  ParseFailed(String),
}
