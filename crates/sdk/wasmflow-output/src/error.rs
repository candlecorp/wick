use vino_transport::error::TransportError;

/// Errors originating from WASM components.
#[derive(Debug)]
pub enum Error {
  /// A serialization or deserialization error.
  Codec(String),

  /// An attempt to take the next packet failed.
  EndOfOutput(String),

  /// Tried to take packets from a port that never produced any.
  PortNotFound(String),

  /// An error originating from a component task.
  Component(String),
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
      Error::Codec(e) => write!(f, "Codec error: {}", e),
      Error::Component(v) => write!(f, "{}", v),
      Error::EndOfOutput(v) => write!(f, "No output available for port '{}'", v),
      Error::PortNotFound(v) => write!(
        f,
        "Tried to take packet from port '{}' but port '{}' never received anything.",
        v, v
      ),
    }
  }
}

impl std::error::Error for Error {}
