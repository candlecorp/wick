use thiserror::Error;

#[derive(Error, Debug, Clone)]

/// The error type for Wasmflow Entities.
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
