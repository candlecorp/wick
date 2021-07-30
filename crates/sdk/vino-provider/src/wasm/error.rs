use vino_codec::error::CodecError;

/// The error type used when attempting to deserialize a [Packet]
#[derive(Debug)]
pub enum Error {
  HostError(String),
  JobNotFound(String),
  CodecError(CodecError),
  MissingInput(String),
}

impl From<CodecError> for Error {
  fn from(e: CodecError) -> Self {
    Error::CodecError(e)
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Invalid")
    // match self {
    //   Error::Invalid => write!(f, "Invalid"),
    //   Error::Exception(v) => write!(f, "{}", v),
    //   Error::Error(v) => write!(f, "{}", v),
    //   Error::DeserializationError(e) => write!(f, "{}", e.to_string()),
    //   Error::InternalError => write!(f, "Internal Error"),
    // }
  }
}

impl std::error::Error for Error {}
