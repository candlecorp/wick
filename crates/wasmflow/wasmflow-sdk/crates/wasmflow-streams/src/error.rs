/// The error type used when attempting to deserialize a [wasmflow_packet::Packet].
#[derive(Debug, Clone, Copy)]
pub enum Error {
  /// Error returned when trying to perform actions on a closed PacketStream.
  Closed,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Closed => write!(f, "Already closed"),
    }
  }
}

impl std::error::Error for Error {}
