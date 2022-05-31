use vino_transport::error::TransportError;
use wasmflow_codec::error::CodecError;

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

  /// Error occurred in the WasmFlow WASM runtime or the protocol between WebAssembly & WasmFlow.
  Protocol(Box<dyn std::error::Error + Send + Sync>),

  /// Error with the async channel.
  ChannelError(String),

  /// Error with the async channel.
  SendChannelClosed,

  /// Could not convert packet to destination type.
  Conversion(String),
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

impl From<TransportError> for Error {
  fn from(e: TransportError) -> Self {
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
      Error::ComponentNotFound(v, valid) => write!(f, "Component '{}' not found. Valid components are: {}", v, valid),
      Error::Codec(e) => write!(f, "Codec error: {}", e),
      Error::MissingInput(v) => write!(f, "Missing input for port '{}'", v),
      Error::Component(v) => write!(f, "{}", v),
      Error::EndOfOutput(v) => write!(f, "No output available for port '{}'", v),
      Error::Protocol(e) => write!(f, "Protocol error: {}", e),
      Error::ChannelError(e) => write!(f, "Error in the async channel: {}", e),
      Error::Conversion(e) => write!(f, "Error converting packet: {}", e),
      Error::SendChannelClosed => write!(f, "Send channel closed"),
    }
  }
}

impl std::error::Error for Error {}
