/// Errors originating from WASM components.
#[derive(Debug)]
pub enum Error {
  /// An error returned from the WaPC host, the system running the WebAssembly module.
  HostError(String),

  /// An error returned from serializing/deserializing a packet or its payload.
  Codec(wasmflow_codec::Error),

  /// Async runtime failure.
  Async,

  /// Error occurred in the WasmFlow WASM runtime or the protocol between WebAssembly & WasmFlow.
  Protocol(Box<dyn std::error::Error + Send + Sync>),

  /// Dispatcher not set before guest call
  DispatcherNotSet,
}

impl From<wasmflow_codec::Error> for Error {
  fn from(v: wasmflow_codec::Error) -> Self {
    Self::Codec(v)
  }
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
      Error::HostError(v) => write!(f, "Error executing host call: {}", v),
      Error::Codec(e) => write!(f, "{}", e),
      Error::Protocol(e) => write!(f, "Protocol error: {}", e),
      Error::Async => write!(f, "Async runtime error"),
      Error::DispatcherNotSet => write!(f, "Dispatcher not set before host call"),
    }
  }
}

impl std::error::Error for Error {}
