/// Wick Manifest's Errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
  /// Tried to specify a directory instead of a configuration file.
  #[error(
    "Can not create a Wick package from directory '{0}'. Please specify a component or application file instead."
  )]
  Directory(String),

  /// Tried to specify a file that is not a component or app file.
  #[error("Can not create a Wick package from {0}. Please specify a component or application file instead.")]
  InvalidWickConfig(String),

  /// Tried to add a resource file that is not in the same directory (or relative subdirectory) as the component or application file.
  #[error("Can not create package with file outside of parent directory scope {0}.")]
  InvalidFileLocation(String),

  /// Failed to parse image reference location
  #[error("Failed to parse the image reference: {0}")]
  InvalidReference(String),

  /// Failed to push package
  #[error("Failed to push the package: {0}")]
  PushFailed(String),

  /// Failed to push package
  #[error("Failed to pull the package: {0}")]
  PullFailed(String),

  /// Failed to create directory
  #[error("Failed to create directory: {0}")]
  DirectoryCreationFailed(String),

  /// Failed to read downloaded package
  #[error("Failed to read downloaded package: {0}")]
  PackageReadFailed(String),
}
