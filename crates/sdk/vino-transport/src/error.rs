use thiserror::Error;

#[derive(Error, Debug)]
/// The crate's Error type
pub enum TransportError {
  /// Error to proxy rmp_serde encoding errors
  #[error("Failed to serialize payload {0}")]
  SerializationError(rmp_serde::encode::Error),
  /// Error to proxy rmp_serde decoding errors
  #[error("Failed to deserialize payload {0}")]
  DeserializationError(rmp_serde::decode::Error),
  /// Error used when converting to a variant fails
  #[error("Payload conversion error")]
  PayloadConversionError(String),
  /// General errors
  #[error("General error : {0}")]
  Other(String),
}
