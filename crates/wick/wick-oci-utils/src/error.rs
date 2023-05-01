use std::path::PathBuf;

use oci_distribution::Reference;
use thiserror::Error;

#[derive(Error, Debug)]
/// Crate error.
pub enum OciError {
  /// No version found in annotations
  #[error("No version found in annotations")]
  NoVersion(),

  /// No manifest found in package
  #[error("No manifest found in package")]
  NoManifest,

  /// Returned when reading an invalid manifest.
  #[error("Invalid manifest found at {}. Try deleting your cache directory.",.0.display())]
  InvalidManifest(PathBuf),

  /// Error thrown when attempting to fetch an image with :latest when forbidden.
  #[error("Configuration disallows fetching artifacts with the :latest tag ({0})")]
  LatestDisallowed(String),

  /// General fetch failure.
  #[error("Could not fetch '{0}': {1}")]
  OciFetchFailure(String, String),

  /// untar failure failure.
  #[error("Could not untar '{0}': {1}")]
  UntarFile(String, String),

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

  /// Failed to parse image reference location
  #[error("Failed to parse the image reference: {0}")]
  InvalidReference(String),

  /// Failed to push package
  #[error("Failed to pull the package: {0}")]
  PullFailed(String),

  /// Tried to pull a layer that didn't include a title.
  #[error("Wick package layers must contain a title")]
  NoTitle,

  /// Error returned when creating directories
  #[error("Failed to create directory '{0}': {1}")]
  CreateDir(PathBuf, #[source] std::io::Error),

  /// Error returned when writing files
  #[error("Failed to write file '{0}': {1}")]
  WriteFile(PathBuf, #[source] std::io::Error),

  /// Tried to publish a component that didn't have a name
  #[error("Published components must be named")]
  NoName,

  /// Tried to pull a layer that didn't include at least one forward slash in the title.
  #[error("Invalid layer path '{0}', layer path must contain at least one forward slash.")]
  InvalidLayerPath(PathBuf),

  /// Failed to read downloaded package
  #[error("Failed to read downloaded package: {0}")]
  PackageReadFailed(String),

  /// Failed to push package
  #[error("Failed to push the package: {0}")]
  PushFailed(String),

  /// Returned when a pull would overwrite existing files and 'overwrite' is not set.
  #[error("Refusing to overwrite {}. Set 'overwrite' to true to force.", .0.iter().map(|v|v.display().to_string()).collect::<Vec<_>>().join(", "))]
  WouldOverwrite(Vec<PathBuf>),
}
