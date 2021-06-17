use thiserror::Error;

// type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

/// Vino Manifest's Errors
#[derive(Error, Debug)]
pub enum ManifestError {
  /// Invalid version found in the parsed manifest
  #[error("Invalid Manifest Version '{0}'")]
  VersionError(String),

  /// Manifest not found at the specified path
  #[error("File not found {0}")]
  FileNotFound(String),

  /// General deserialization error
  #[error("Failed to deserialize configuration {0}")]
  ConfigurationDeserialization(String),

  /// IO Error
  #[error(transparent)]
  IOError(#[from] std::io::Error),

  /// Error deserializing HOCON manifest
  #[error(transparent)]
  HoconError(#[from] hocon::Error),

  /// Error deserializing YAML manifest
  #[error(transparent)]
  YamlError(#[from] serde_yaml::Error),

  /// Miscellaneous error
  #[error("General error : {0}")]
  Other(String),
}
