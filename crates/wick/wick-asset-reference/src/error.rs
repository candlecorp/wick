use thiserror::Error;

/// Wick Manifest's Errors.
#[derive(Error, Debug)]
pub enum Error {
  /// Location reference was not a URL or package reference
  #[error("Could not parse {0} as a URL or reference")]
  BadUrl(String),

  /// Other URL Parsing error.
  #[error(transparent)]
  UrlParse(#[from] url::ParseError),

  /// Could not load file.
  #[error("Could not read file {0}: {1}")]
  LoadError(String, String),

  /// IP address in manifest is invalid.
  #[error("Invalid IP Address: {0}")]
  BadIpAddress(String),

  /// Path normalization failed.
  #[error("Normalizing path with baseurl failed '{0}': {1}")]
  BaseUrlFailure(String, String),
}
