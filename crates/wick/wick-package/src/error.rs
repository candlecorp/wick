use std::path::PathBuf;

/// This crate's primary Error type.
#[derive(thiserror::Error, Debug)]
pub enum Error {
  /// Tried to specify a directory instead of a configuration file.
  #[error(
    "Can not create a Wick package from directory '{0}'. Please specify a component or application file instead."
  )]
  Directory(PathBuf),

  /// Tried to specify a file that is not a component or app file.
  #[error("Can not create a Wick package from {0}. Please specify a component or application file instead.")]
  InvalidWickConfig(String),

  /// Tried to add a resource file that is not in the same directory (or relative subdirectory) as the component or application file.
  #[error("Can not create package with file outside of parent directory scope {0}.")]
  InvalidFileLocation(String),

  /// Target directory not found or not readable.
  #[error("Target directory {0} not found or not readable.")]
  DestinationDir(String),

  /// A manifest included a reference to a file that could not be found on disk.
  #[error("Can not find file at {0}.")]
  NotFound(String),

  /// Failed to read downloaded package
  #[error("Failed to read downloaded package: {0}")]
  PackageReadFailed(String),

  /// Error returned when reading a file
  #[error("Failed to read file '{0}': {1}")]
  ReadFile(PathBuf, #[source] std::io::Error),

  /// Error returned when working with tar files
  #[error("Failed to read file '{0}': {1}")]
  TarFile(String, #[source] std::io::Error),

  /// Error returned when working with gz files
  #[error("Failed to read file '{0}': {1}")]
  GzipFile(PathBuf, #[source] std::io::Error),

  /// Error returned when working with gz files
  #[error("Error in gzip: {0}")]
  GzipFailed(#[source] std::io::Error),

  /// Tried to publish a component that didn't have a name
  #[error("Published components must be named")]
  NoName,

  /// General Configuration error
  #[error(transparent)]
  Config(#[from] wick_config::Error),

  /// Errors related to OCI push/pull
  #[error(transparent)]
  Oci(#[from] wick_oci_utils::Error),

  /// General asset error
  #[error(transparent)]
  AssetReference(#[from] wick_config::AssetError),

  /// Could not parse contents as JSON
  #[error("Could not parse {0} as JSON: {1}")]
  InvalidJson(&'static str, #[source] serde_json::Error),
}
