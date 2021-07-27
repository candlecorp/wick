use thiserror::Error;

#[derive(Error, Debug)]
/// Crate error
pub enum OciError {
  /// Error thrown when attempting to fetch an image with :latest when forbidden
  #[error("Configuration disallows fetching artifacts with the :latest tag ({0})")]
  LatestDisallowed(String),

  /// General fetch failure
  #[error("Could not fetch '{0}': {1}")]
  OciFetchFailure(String, String),

  /// Error for invalid URLs
  #[error("Could not parse OCI URL {0}: {1}")]
  OCIParseError(String, String),

  /// IO error for the local cache
  #[error(transparent)]
  IOError(#[from] std::io::Error),
}
