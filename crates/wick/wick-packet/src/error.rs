use serde_json::Value;
use wick_interface_types::Type;

use crate::PacketError;

/// Errors originating from WASM components.
#[derive(Debug, thiserror::Error, PartialEq, Clone)]
#[non_exhaustive]
pub enum Error {
  /// Thrown when a user attempts to retrieve a stream for a port that doesn't exist.
  #[error("No stream found for port '{0}'")]
  PortMissing(String),

  /// Error deserializing payload.
  #[error("Error deserializing payload '{}': {}",.as_json,.error)]
  Decode { as_json: String, error: String },

  /// Error converting payload into JSON.
  #[error("Error JSON-ifying payload: {0}")]
  Jsonify(String),

  /// Error communicating over a stream or channel.
  #[error("Error communicating over a stream or channel: {0}")]
  Channel(String),

  /// General error to wrap other errors.
  #[error("{0}")]
  Component(String),

  /// Payload was successful but no data was provided.
  #[error("No data in payload")]
  NoData,

  /// An error that wraps a PayloadError.
  #[error("{}", .0.msg())]
  PayloadError(PacketError),

  /// Thrown when a user attempts to use a signal when they expected a payload.
  #[error("Got a Done signal in an unexpected context.")]
  UnexpectedDone,

  /// Returned when an operation attempts to retrieve a configuration item that doesn't exist or decoding fails.
  #[error("Could not retrieve configuration item '{0}'")]
  ContextKey(String),

  /// Returned when trying to decode a non-JSON object into [crate::RuntimeConfig].
  #[error("Can only convert JSON Objects to a operation and component configuration, got '{0}'")]
  BadJson(Value),

  /// Couldn't retrieve a complete set of packets from a [crate::StreamMap]
  #[error("Could not retrieve a complete set of packets. Stream '{0}' failed to provide a packet: '{1}'")]
  StreamMapError(String /* port */, String /* error */),

  /// Couldn't retrieve a complete set of packets from a [crate::StreamMap]
  #[error("Could not retrieve a complete set of packets. Stream '{0}' completed or failed before providing a packet.")]
  StreamMapMissing(String /* port */),

  #[error("Configuration provided for component '{0}' does not match expected signature, {1}")]
  Signature(String, String),

  #[cfg(feature = "datetime")]
  #[error("Error parsing date '{0}', date must be an RFC 3339 formatted string")]
  ParseDate(String),

  #[cfg(feature = "datetime")]
  #[error("Error parsing date '{0}', date must be milliseconds from the UNIX epoch")]
  ParseDateMillis(u64),

  #[error("Could not coerce value {value} to a {desired}")]
  Coersion { value: Value, desired: Type },
}

impl Error {
  pub fn component_error<T: Into<String>>(msg: T) -> Self {
    Self::Component(msg.into())
  }
}

impl From<wasmrs_rx::Error> for Error {
  fn from(value: wasmrs_rx::Error) -> Self {
    Self::Channel(value.to_string())
  }
}

impl From<Box<dyn std::error::Error>> for Error {
  fn from(value: Box<dyn std::error::Error>) -> Self {
    Self::Component(value.to_string())
  }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
/// The error type for Wick Entities.
pub enum ParseError {
  /// Encountered an invalid scheme when parsing an entity URL.
  #[error("Invalid scheme {0}")]
  Scheme(String),
  /// No path supplied in the entity URL.
  #[error("Missing path")]
  MissingPath,
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
  /// Error converting arguments into an [crate::Entity].
  #[error(transparent)]
  Conversion(Box<dyn std::error::Error + Send + Sync>),
}
