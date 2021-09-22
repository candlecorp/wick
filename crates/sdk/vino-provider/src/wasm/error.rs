use vino_codec::error::CodecError;

/// Errors originating from WASM components.
#[derive(Debug)]
pub enum Error {
  /// An error returned from the WaPC host, the system running the WebAssembly module.
  HostError(String),

  /// A serialization or deserialization error.
  CodecError(String),

  /// The requested component was not found in this module.
  ComponentNotFound(String, String),

  /// An input the component expects was not found.
  MissingInput(String),

  /// An error originating from a component task.
  Component(String),
}

#[derive(Debug)]
/// Error originating from a component task.
pub struct ComponentError(String);

impl ComponentError {
  /// Constructor for a [ComponentError].
  pub fn new<T: AsRef<str>>(message: T) -> Self {
    Self(message.as_ref().to_owned())
  }
}

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

impl From<CodecError> for Error {
  fn from(e: CodecError) -> Self {
    Error::CodecError(e.to_string())
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let msg = match self {
      Error::HostError(v) => format!("Host error: {}", v),
      Error::ComponentNotFound(v, valid) => format!(
        "Component '{}' not found. Valid components are: {}",
        v, valid
      ),
      Error::CodecError(e) => format!("Codec error: {}", e),
      Error::MissingInput(v) => format!("Missing input for port '{}'", v),
      Error::Component(v) => format!("Component error: '{}'", v),
    };
    write!(f, "{}", msg)
  }
}

impl std::error::Error for Error {}
