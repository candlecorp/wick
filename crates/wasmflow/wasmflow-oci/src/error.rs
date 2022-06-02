use oci_distribution::Reference;
use thiserror::Error;

#[derive(Error, Debug)]
/// Crate error.
pub enum OciError {
  /// Error thrown when attempting to fetch an image with :latest when forbidden.
  #[error("Configuration disallows fetching artifacts with the :latest tag ({0})")]
  LatestDisallowed(String),

  /// General fetch failure.
  #[error("Could not fetch '{0}': {1}")]
  OciFetchFailure(String, String),

  /// Failure during OCI push.
  #[error("Could not push '{0}': {1}")]
  OciPushFailure(Reference, Box<dyn std::error::Error + Send + Sync>),

  /// Failure during OCI push.
  #[error("Could not push manifest list '{0}': {1}")]
  OciPushManifestListFailure(Reference, Box<dyn std::error::Error + Send + Sync>),

  /// Failed to retrieve a manifest.
  #[error("Could not retrieve manifest for '{0}': {1}")]
  OciPullManifestFailure(Reference, Box<dyn std::error::Error + Send + Sync>),

  /// Error for invalid URLs.
  #[error("Could not parse OCI URL {0}: {1}")]
  OCIParseError(String, String),

  /// IO error for the local cache.
  #[error(transparent)]
  IOError(#[from] std::io::Error),

  /// Upstream errors from oci-distribution
  #[error(transparent)]
  OciDistribution(#[from] oci_distribution::errors::OciDistributionError),

  /// JSON Parse Error
  #[error(transparent)]
  JsonParseFailed(#[from] serde_json::Error),

  /// YAML Parse Error
  #[error(transparent)]
  YamlParseFailed(#[from] serde_yaml::Error),

  /// Wascap Error
  #[error(transparent)]
  WasCap(#[from] wasmflow_wascap::Error),

  /// Provider Archive Error
  #[error(transparent)]
  Par(#[from] wasmflow_par::Error),
}
