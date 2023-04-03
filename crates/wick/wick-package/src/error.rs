use std::path::PathBuf;

/// Wick Manifest's Errors.
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

  /// Tried to pull a layer that didn't include at least one forward slash in the title.
  #[error("Invalid layer path '{0}', layer path must contain at least one forward slash.")]
  InvalidLayerPath(PathBuf),

  /// Tried to pull a layer that didn't include a title.
  #[error("Wick package layers must contain a title")]
  NoTitle,

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

  /// Error returned when reading a file
  #[error("Failed to read file '{0}': {1}")]
  ReadFile(PathBuf, #[source] std::io::Error),

  /// Error returned when creating directories
  #[error("Failed to create directory '{0}': {1}")]
  CreateDir(PathBuf, #[source] std::io::Error),

  /// Error returned when writing files
  #[error("Failed to write file '{0}': {1}")]
  WriteFile(PathBuf, #[source] std::io::Error),

  /// Tried to publish a component that didn't have a name
  #[error("Published components must be named")]
  NoName,

  /// Tried to pull a package that had no tag.
  #[error("Package has no tag")]
  NoTag,

  /// General Configuration error
  #[error(transparent)]
  Config(#[from] wick_config::Error),

  /// General URL conversion or parsing error
  #[error(transparent)]
  Url(#[from] url::ParseError),

  /// Could not parse contents as JSON
  #[error("Could not parse {0} as JSON: {1}")]
  InvalidJson(&'static str, #[source] serde_json::Error),
}
