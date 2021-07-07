use thiserror::Error;

#[derive(Error, Debug)]

/// The error type for Vino Entities.
pub enum EntityError {
  /// Error used when trying to parse a URL into an entity.
  #[error("URL parse error {0}")]
  ParseError(String),
  /// Error when converting an Entity into a variant without checking first.
  #[error("Conversion error {0}")]
  ConversionError(&'static str),
}
