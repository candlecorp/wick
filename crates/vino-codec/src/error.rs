use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodecError {
  #[error("Failed to serialize payload {0}")]
  SerializationError(rmp_serde::encode::Error),
  #[error("Failed to deserialize payload {0}")]
  DeserializationError(rmp_serde::decode::Error),

  #[error("General error : {0}")]
  Other(String),
}
