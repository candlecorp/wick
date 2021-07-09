use std::string::FromUtf8Error;

use parity_wasm::SerializationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClaimsError {
  #[error("Invalid module hash")]
  InvalidModuleHash,
  #[error(transparent)]
  Utf8Error(#[from] FromUtf8Error),
  #[error(transparent)]
  WascapError(#[from] wascap::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  SerializationError(#[from] SerializationError),
  #[error("General error : {0}")]
  Other(String),
}
