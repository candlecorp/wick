use std::path::PathBuf;

use thiserror::Error;

/// This crate's primary Error type.
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
  /// Location reference was not a URL or package reference
  #[error("Could not parse {0} as a URL or reference")]
  BadUrl(String),

  /// Could not load file.
  #[error("Could not read file {}", .0.display())]
  LoadError(PathBuf),

  /// Remote fetch failed.
  #[error("Asset {} wasn't found locally and couldn't be pulled: {1}", .0.display())]
  PullFailed(PathBuf, String),

  /// Local path normalization failed.
  #[error("Normalization failed for path '{0}': {1}")]
  NormalizationFailure(String, String),

  /// Could not find file or directory.
  #[error("File or directory {} not found", .0.display())]
  NotFound(PathBuf),

  /// Could not resolve a location to a filesystem path or an OCI reference.
  #[error("Could not resolve {0} to a filesystem location or an OCI reference")]
  Unresolvable(String),

  /// Error returned when a file path does not reside in a target directory.
  #[error("File {} does not reside in target directory {1}", .0.display())]
  FileEscapesRoot(PathBuf, String),
}
