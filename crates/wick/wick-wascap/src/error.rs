use std::str::Utf8Error;

// use parity_wasm::SerializationError;
use thiserror::Error;
use wasmparser::BinaryReaderError;

#[derive(Error, Debug)]
#[non_exhaustive]
/// Wick WasCap's error type.
pub enum Error {
  #[error("Invalid module hash")]
  /// Error returned when a module's hash does not match the hash embedded in its token.
  InvalidModuleHash,

  #[error("Invalid module format, the 'jwt' custom section is missing")]
  /// Error returned when we could not find the module's JWT section
  InvalidModuleFormat,

  #[error(transparent)]
  /// Error parsing string.
  Utf8Error(#[from] Utf8Error),

  #[error(transparent)]
  /// Error reading a buffer.
  IoError(#[from] std::io::Error),

  #[error(transparent)]
  /// Error reading a buffer.
  ParserReadError(#[from] BinaryReaderError),

  // #[error(transparent)]
  /// Error injecting token into WebAssembly module.
  // SerializationError(#[from] SerializationError),
  #[error("Parse error for wasm module: {0}")]
  ParseError(String),

  #[error("General error : {0}")]
  /// General error.
  Other(String),

  /// Invalid token format
  #[error("Invalid token format")]
  Token,

  /// Error parsing JSON
  #[error(transparent)]
  Json(#[from] serde_json::Error),

  /// Error decoding base64
  #[error(transparent)]
  Base64(#[from] base64::DecodeError),

  /// Error signing token
  #[error(transparent)]
  Sign(#[from] nkeys::error::Error),

  /// Error decoding utf8 bytes
  #[error("{0} not utf8 bytes")]
  Utf8(String),

  /// Token is not yet valid
  #[error("Token is not yet valid")]
  TokenTooEarly,

  /// Token has expired
  #[error("Token has expired")]
  ExpiredToken,

  /// Token is missing an issuer
  #[error("Token is missing an issuer")]
  MissingIssuer,

  /// Token is missing a subject
  #[error("Token is missing a subject")]
  MissingSubject,

  /// Token uses an invalid algorithm
  #[error("Token uses an invalid algorithm")]
  InvalidAlgorithm,
}
