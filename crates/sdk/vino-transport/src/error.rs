use thiserror::Error;

#[derive(Error, Debug, Clone)]

/// The public Error type for [crate::MessageTransport] values.
/// These errors signify a held error or an error transforming
/// a [crate::MessageTransport] into another type.
pub enum TransportError {
  /// Error to proxy codec errors.
  #[error("Serialization error: {0}")]
  SerializationError(String),

  /// Error to proxy decoding errors.
  #[error("Deserialization error: {0}")]
  DeserializationError(String),

  /// Error used when a payload is invalid or invalidated.
  #[error("Invalid payload")]
  Invalid,

  /// Error from the actual payload.
  #[error("{0}")]
  Error(String),

  /// Exception from the actual payload.
  #[error("{0}")]
  Exception(String),

  /// General errors.
  #[error("General error : {0}")]
  Other(String),
}
