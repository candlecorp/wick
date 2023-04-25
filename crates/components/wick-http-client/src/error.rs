use flow_component::ComponentError;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
  #[error("Invalid output for operations {}. At this time postgres operations can have at most one output named 'output' of type 'object'", .0.join(", "))]
  InvalidOutput(Vec<String>),

  #[error("Bad configuration: {0}")]
  Validation(String),

  #[error("Failed to prepare arguments: {0}")]
  Prepare(String),

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
