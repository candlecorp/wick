use thiserror::Error;

#[derive(Error, Debug)]
pub enum OciError {
  #[error("Configuration disallows fetching artifacts with the :latest tag ({0})")]
  LatestDisallowed(String),
  #[error("Could not fetch '{0}': {1}")]
  OciFetchFailure(String, String),
  #[error("Could not parse OCI URL {0}: {1}")]
  OCIParseError(String, String),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
}
