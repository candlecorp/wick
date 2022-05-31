use thiserror::Error;

#[derive(Error, Debug)]

/// The public Error type for [crate::MessageTransport] values.
/// These errors signify a held error or an error transforming
/// a [crate::MessageTransport] into another type.
pub enum TransportError {
  /// Error to proxy codec errors.
  #[error("De/serialization error: {0}")]
  Codec(#[from] wasmflow_codec::Error),

  /// Error to proxy decoding errors.
  #[error("Deserialization error: {0}")]
  DeserializationError(String),

  /// Error deserializing incoming payload.
  #[error("Error deserializing incoming payload: {0}")]
  IncomingPayload(String),

  /// Error used when a payload is invalid or invalidated.
  #[error("Invalid payload")]
  Invalid,

  /// Error used when a payload is invalid or invalidated.
  #[error("Payload was an internal signal and should not be deserialized")]
  Signal,

  /// Error from the actual payload.
  #[error("{0}")]
  Error(String),

  /// Exception from the actual payload.
  #[error("{0}")]
  Exception(String),

  /// Error returned when a stream is collected when already consumed.
  #[error("Stream has already been collected")]
  AlreadyCollected,

  /// General errors.
  #[error("General error : {0}")]
  Other(String),
}

impl From<serde_json::Error> for TransportError {
  fn from(v: serde_json::Error) -> Self {
    Self::DeserializationError(v.to_string())
  }
}
