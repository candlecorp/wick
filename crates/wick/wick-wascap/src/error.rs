use std::string::FromUtf8Error;

// use parity_wasm::SerializationError;
use thiserror::Error;

#[derive(Error, Debug)]
/// Wick WasCap's error type.
pub enum ClaimsError {
  #[error("Invalid module hash")]
  /// Error returned when a module's hash does not match the hash embedded in its token.
  InvalidModuleHash,

  #[error(transparent)]
  /// Error parsing string.
  Utf8Error(#[from] FromUtf8Error),

  #[error(transparent)]
  /// Upstream error from [wascap].
  WascapError(#[from] wascap::Error),

  #[error(transparent)]
  /// Error reading a buffer.
  IoError(#[from] std::io::Error),

  // #[error(transparent)]
  /// Error injecting token into WebAssembly module.
  // SerializationError(#[from] SerializationError),
  #[error("Parse error for wasm module: {0}")]
  ParseError(String),

  #[error("General error : {0}")]
  /// General error.
  Other(String),
}
