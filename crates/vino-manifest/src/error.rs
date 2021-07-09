use thiserror::Error;

use crate::{
  ConnectionDefinition,
  ConnectionTargetDefinition,
};

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

  /// Component id is not a fully qualified name with a namespace
  #[error("Component id '{0}' is not a fully qualified name with a namespace")]
  ComponentIdError(String),

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

  /// Default was requested when none present
  #[error("Connection '{0}' does not have a default but one was requested")]
  NoDefault(ConnectionDefinition),

  /// Error deserializing default value
  #[error("Error deserializing default value for connection: {0}=>{1} - Error was: '{2}'")]
  DefaultsError(
    ConnectionTargetDefinition,
    ConnectionTargetDefinition,
    serde_json::Error,
  ),

  /// Miscellaneous error
  #[error("General error : {0}")]
  Other(String),
}
