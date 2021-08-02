/// The error type used when attempting to deserialize a [Packet]
#[derive(Debug)]
pub enum DeserializationError {
  /// Invalid payload
  Invalid,
  /// Packet was an Exception
  Exception(String),
  /// Packet was an Error
  Error(String),
  /// An error deserializing from MessagePack
  DeserializationError(vino_codec::Error),
  /// An Internal error given when the packet contained a message not destined for a consumer
  InternalError,
}

impl std::fmt::Display for DeserializationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DeserializationError::Invalid => write!(f, "Refused to deserialize invalid payload"),
      DeserializationError::Exception(v) => write!(f, "Exception: {}", v),
      DeserializationError::Error(v) => write!(f, "Error: {}", v),
      DeserializationError::DeserializationError(e) => {
        write!(f, "Deserialization Error: {}", e.to_string())
      }
      DeserializationError::InternalError => write!(f, "Internal Deserialization Error"),
    }
  }
}

impl std::error::Error for DeserializationError {}
