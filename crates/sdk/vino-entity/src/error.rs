use thiserror::Error;

#[derive(Error, Debug)]

/// The error type for Vino Entities.
pub enum EntityError {
  /// Error used when trying to parse a URL into an entity.
  #[error("URL parse error {0}")]
  ParseError(String),
}
