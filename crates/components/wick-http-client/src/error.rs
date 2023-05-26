use flow_component::ComponentError;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
  #[error("Bad configuration: {0}")]
  Validation(String),

  #[error("Failed to render path template {0}: {1}")]
  PathTemplate(String, String),

  #[error("Received invalid header value for header {0}")]
  InvalidHeader(String),

  #[error("Could not find operation {0} on this component")]
  OpNotFound(String),
}

impl From<Error> for ComponentError {
  fn from(value: Error) -> Self {
    ComponentError::new(value)
  }
}
