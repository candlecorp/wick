/// Errors originating from WASM components.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// Error deserializing incoming payload.
  #[error("Error deserializing incoming payload: {0}")]
  IncomingPayload(String),
}
