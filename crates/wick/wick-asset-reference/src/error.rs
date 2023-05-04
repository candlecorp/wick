use std::path::PathBuf;

use thiserror::Error;

/// This crate's primary Error type.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
  /// Location reference was not a URL or package reference
  #[error("Could not parse {0} as a URL or reference")]
  BadUrl(String),

  /// Could not load file.
  #[error("Could not read file {}: {1}", .0.display())]
  LoadError(PathBuf, String),

  /// Could not find file during path normalization.
  #[error("Could not read file at path {0}: {1}")]
  NormalizationFailure(String, String),

  /// Could not find file or directory.
  #[error("File or directory {} not found", .0.display())]
  NotFound(PathBuf),

  /// Error returned when a file path does not reside in a target directory.
  #[error("File {} does not reside in target directory {1}", .0.display())]
  FileEscapesRoot(PathBuf, String),
}
