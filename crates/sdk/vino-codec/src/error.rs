use thiserror::Error;

#[derive(Error, Debug)]
/// vino-codec's Error type.
pub enum CodecError {
  /// Error to proxy rmp_serde encoding errors.
  #[error("Failed to serialize payload into MessagePack: {0}")]
  MessagePackSerializationError(rmp_serde::encode::Error),
  /// Error to proxy rmp_serde decoding errors.
  #[error("Failed to deserialize MessagePack payload: {0}")]
  MessagePackDeserializationError(rmp_serde::decode::Error),
  /// Error to proxy serde_json encoding errors.
  #[error("Failed to serialize payload into JSON: {0}")]
  JsonSerializationError(serde_json::Error),
  /// Error to proxy serde_json decoding errors.
  #[error("Failed to deserialize JSON payload: {0}")]
  JsonDeserializationError(serde_json::Error),
  /// Error when serializing to a raw value.
  #[error("Failed to serialize payload: {0}")]
  SerializationError(serde_value::SerializerError),
  /// Error when deserialization from a raw value.
  #[error("Failed to deserialize payload: {0}")]
  DeserializationError(serde_value::DeserializerError),
  /// Error returned when requesting a field of the payload that doesn't exist.
  #[error("Input data for port '{0}' missing")]
  MissingInput(String),

  #[doc(hidden)]
  #[error("General error : {0}")]
  Other(String),
}
