use vino_codec::error::CodecError;

/// The error type used when attempting to deserialize a [vino_packet::Packet].
#[derive(Debug)]
pub enum Error {
  /// An error returned from the WaPC host, the system running the WebAssembly module.
  HostError(String),

  /// A serialization or deserialization error.
  CodecError(CodecError),

  /// The requested component was not found in this module.
  ComponentNotFound(String, String),

  /// An input the component expects was not found.
  MissingInput(String),
}

impl From<CodecError> for Error {
  fn from(e: CodecError) -> Self {
    Error::CodecError(e)
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::HostError(v) => write!(f, "Host error: {}", v),
      Error::ComponentNotFound(v, valid) => write!(
        f,
        "Component '{}' not found. Valid components are: {}",
        v, valid
      ),
      Error::CodecError(e) => write!(f, "Codec error: {}", e),
      Error::MissingInput(v) => write!(f, "Missing input for port '{}'", v),
    }
  }
}

impl std::error::Error for Error {}
