use thiserror::Error;

#[derive(Error, Debug)]
pub enum EntityError {
  #[error("URL parse error {0}")]
  ParseError(String),
  #[error("Conversion error {0}")]
  ConversionError(&'static str),
}
