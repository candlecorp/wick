use vino_codec::error::CodecError;

/// The error type used when attempting to deserialize a [Packet]
#[derive(Debug)]
pub enum Error {
  HostError(String),
  CodecError(CodecError),
  JobNotFound(String),
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
      Error::HostError(v) => write!(f, "Host Error: {}", v),
      Error::JobNotFound(v) => write!(f, "Component not found: {}", v),
      Error::CodecError(e) => write!(f, "Codec Error: {}", e),
      Error::MissingInput(v) => write!(f, "Missing Input: {}", v),
    }
  }
}

impl std::error::Error for Error {}
