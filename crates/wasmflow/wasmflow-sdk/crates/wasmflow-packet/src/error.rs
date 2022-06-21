/// The error type used when attempting to deserialize a [crate::Packet].
#[derive(Debug)]
pub enum Error {
  /// Tried to deserialize a Signal packet.
  Signal,
  /// Invalid payload.
  Invalid,
  /// Packet was an Exception.
  Exception(String),
  /// Packet was an Error.
  Error(String),
  /// An error deserializing from MessagePack.
  DeserializationError(wasmflow_codec::Error),
  /// An Internal error given when the packet contained a message not destined for a consumer.
  InternalError,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Signal => write!(f, "Tried to deserialize a Signal packet"),
      Error::Invalid => write!(f, "Refused to deserialize invalid payload"),
      Error::Exception(v) => write!(f, "Exception: {}", v),
      Error::Error(v) => write!(f, "Error: {}", v),
      Error::DeserializationError(e) => {
        write!(f, "Deserialization Error: {}", e)
      }
      Error::InternalError => write!(f, "Internal Deserialization Error"),
    }
  }
}

impl std::error::Error for Error {}
