use thiserror::Error;

/// The RPC Error type
#[derive(Error, Debug)]
pub enum Error {
  /// Error during the parsing of an IP address and port
  #[error(transparent)]
  AddrParseError(#[from] std::net::AddrParseError),
  /// General Error
  #[error("General error : {0}")]
  Other(String),
}
