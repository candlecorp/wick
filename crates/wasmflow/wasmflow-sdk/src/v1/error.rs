pub use crate::v1::BoxedError;

#[derive(Debug)]
pub enum Error {
  /// An input the component expects was not found.
  MissingInput(String),

  /// An error from an upstream module.
  Upstream(Box<dyn std::error::Error + Send + Sync>),

  /// Error sending packet to output port.
  SendError(String),

  /// The requested component was not found in this module.
  ComponentNotFound(String, String),

  /// An error resulting from deserializing or serializing a payload.
  CodecError(String),
  /// culling line
  /// An error returned from the WaPC host, the system running the WebAssembly module.
  HostError(String),

  /// Async runtime failure.
  Async,

  /// Dispatcher not set before guest call
  DispatcherNotSet,
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

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::ComponentNotFound(v, valid) => write!(f, "Component '{}' not found. Valid components are: {}", v, valid),
      Error::Upstream(v) => write!(f, "{}", v),
      Error::MissingInput(v) => write!(f, "Missing input for port '{}'", v),
      Error::SendError(port) => write!(f, "Error sending packet to output port '{}'", port),
      Error::CodecError(v) => write!(f, "{}", v),
      Error::HostError(v) => write!(f, "Error executing host call: {}", v),
      Error::Async => write!(f, "Async runtime error"),
      Error::DispatcherNotSet => write!(f, "Dispatcher not set before host call"),
    }
  }
}

impl std::error::Error for Error {}

impl From<wasmflow_packet::error::Error> for Error {
  fn from(e: wasmflow_packet::error::Error) -> Self {
    Error::Upstream(Box::new(e))
  }
}

impl From<wasmflow_transport::Error> for Error {
  fn from(e: wasmflow_transport::Error) -> Self {
    Error::Upstream(Box::new(e))
  }
}

impl From<wasmflow_output::error::Error> for Error {
  fn from(e: wasmflow_output::error::Error) -> Self {
    Error::Upstream(Box::new(e))
  }
}

impl From<wasmflow_codec::Error> for Error {
  fn from(e: wasmflow_codec::Error) -> Self {
    Error::CodecError(e.to_string())
  }
}

impl From<wasmflow_entity::Error> for Error {
  fn from(e: wasmflow_entity::Error) -> Self {
    Error::Upstream(Box::new(e))
  }
}

impl From<BoxedError> for Error {
  fn from(e: BoxedError) -> Self {
    Error::Upstream(e)
  }
}
