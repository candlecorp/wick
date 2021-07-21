use thiserror::Error;

#[derive(Error, Debug)]
/// vino-codec's Error type
pub enum CodecError {
  /// Error to proxy rmp_serde encoding errors
  #[error("Failed to serialize payload {0}")]
  SerializationError(rmp_serde::encode::Error),
  /// Error to proxy rmp_serde decoding errors
  #[error("Failed to deserialize payload {0}")]
  DeserializationError(rmp_serde::decode::Error),
  /// Error to proxy rmp_serde decoding errors
  #[error("Input data for port '{0}' missing")]
  MissingInput(String),

  #[doc(hidden)]
  #[error("General error : {0}")]
  Other(String),
}
