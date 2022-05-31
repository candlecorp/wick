/// Errors originating from WASM components.
#[derive(Debug)]
pub enum Error {
  /// A serialization or deserialization error.
  Codec(String),

  /// The requested component was not found in this module.
  ComponentNotFound(String, String),

  /// An input the component expects was not found.
  MissingInput(String),

  /// An attempt to take the next packet failed.
  EndOfOutput(String),

  /// An error originating from a component task.
  Component(String),

  /// An attempt to take packets for a port failed because no packets were found.
  ResponseMissing(String),
}

#[derive(Debug)]
/// Error originating from a component task.
pub struct ComponentError(String);

impl ComponentError {
  /// Constructor for a [ComponentError].
  pub fn new<T: std::fmt::Display>(message: T) -> Self {
    Self(message.to_string())
  }
}

impl std::error::Error for ComponentError {}

impl std::fmt::Display for ComponentError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<ComponentError> for Error {
  fn from(e: ComponentError) -> Self {
    Self::Component(e.to_string())
  }
}

impl From<String> for Error {
  fn from(e: String) -> Self {
    Error::Component(e)
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::ComponentNotFound(v, valid) => write!(f, "Component '{}' not found. Valid components are: {}", v, valid),
      Error::Codec(e) => write!(f, "Codec error: {}", e),
      Error::MissingInput(v) => write!(f, "Missing input for port '{}'", v),
      Error::Component(v) => write!(f, "{}", v),
      Error::EndOfOutput(v) => write!(f, "No output available for port '{}'", v),
      Error::ResponseMissing(v) => write!(f, "No response received for port '{}'", v),
    }
  }
}

impl std::error::Error for Error {}
