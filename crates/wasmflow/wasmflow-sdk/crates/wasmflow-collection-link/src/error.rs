use wasmflow_codec::error::CodecError;

/// Errors originating from WASM components.
#[derive(Debug)]
pub enum Error {
  /// A serialization or deserialization error.
  Codec(String),

  /// An error originating from a component task.
  Component(String),

  /// Error occurred in the WasmFlow WASM runtime or the protocol between WebAssembly & WasmFlow.
  Protocol(Box<dyn std::error::Error + Send + Sync>),
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

impl From<CodecError> for Error {
  fn from(e: CodecError) -> Self {
    Error::Codec(e.to_string())
  }
}

impl From<&str> for Error {
  fn from(e: &str) -> Self {
    Error::Component(e.to_owned())
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
      Error::Codec(e) => write!(f, "Codec error: {}", e),
      Error::Component(v) => write!(f, "{}", v),
      Error::Protocol(e) => write!(f, "Protocol error: {}", e),
    }
  }
}

impl std::error::Error for Error {}
