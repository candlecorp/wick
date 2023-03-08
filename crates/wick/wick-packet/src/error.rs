use crate::PacketError;

/// Errors originating from WASM components.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// Thrown when a user attempts to retrieve a stream for a port that doesn't exist.
  #[error("No stream found for port '{0}'")]
  PortMissing(String),

  /// Error serializing or deserializing payload.
  #[error("Error serializing or deserializing payload: {0}")]
  Codec(String),

  /// Error communicating over a stream or channel.
  #[error("Error communicating over a stream or channel: {0}")]
  Channel(String),

  /// General error to wrap other errors.
  #[error("{0}")]
  General(String),

  /// An error that wraps a PayloadError.
  #[error("{}", .0.msg())]
  PayloadError(PacketError),

  /// Thrown when a user attempts to use a signal when they expected a payload.
  #[error("Got a Done signal in an unexpected context.")]
  UnexpectedDone,
}

impl From<wasmrs_codec::error::Error> for Error {
  fn from(value: wasmrs_codec::error::Error) -> Self {
    Self::Codec(value.to_string())
  }
}

impl From<wasmrs_rx::Error> for Error {
  fn from(value: wasmrs_rx::Error) -> Self {
    Self::Channel(value.to_string())
  }
}

impl From<Box<dyn std::error::Error>> for Error {
  fn from(value: Box<dyn std::error::Error>) -> Self {
    Self::General(value.to_string())
  }
}

#[derive(thiserror::Error, Debug, Clone)]

/// The error type for Wick Entities.
pub enum ParseError {
  /// Encountered an invalid scheme when parsing an entity URL.
  #[error("Invalid scheme {0}")]
  Scheme(String),
  /// No authority/host supplied in the entity URL.
  #[error("Missing authority/host")]
  Authority,
  /// Invalid authority/host supplied in the entity URL.
  #[error("Invalid authority/host '{0}', missing separator '.'")]
  InvalidAuthority(String),
  /// Invalid authority/host kind.
  #[error("Invalid authority/host kind '{0}'")]
  InvalidAuthorityKind(String),
  /// Error parsing an entity URL.
  #[error("{0}")]
  Parse(url::ParseError),
}
