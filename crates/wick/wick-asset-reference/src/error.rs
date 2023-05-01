use thiserror::Error;

/// This crate's primary Error type.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
  /// Location reference was not a URL or package reference
  #[error("Could not parse {0} as a URL or reference")]
  BadUrl(String),

  /// Could not load file.
  #[error("Could not read file {0}: {1}")]
  LoadError(String, String),

  /// Path normalization failed.
  #[error("Could not normalize path {0}: {1}")]
  BaseUrlFailure(String, String),

  /// Could not find file or directory.
  #[error("File or directory {0} not found")]
  NotFound(String),

  /// Error returned when a file path does not reside in a target directory.
  #[error("File {0} does not reside in target directory {1}")]
  FileEscapesRoot(String, String),
}
