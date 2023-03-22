use thiserror::Error;

use crate::{ConnectionDefinition, ConnectionTargetDefinition};

// type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

/// Wick Manifest's Errors.
#[derive(Error, Debug)]
pub enum ManifestError {
  /// Invalid version found in the parsed manifest.
  #[error("Invalid Manifest Version '{0}'")]
  VersionError(String),

  /// No version found in the parsed manifest.
  #[error("Manifest needs a format version")]
  NoFormat,

  /// Manifest not found at the specified path.
  #[error("File not found {0}")]
  FileNotFound(String),

  /// Could not load file.
  #[error("Could not read file {0}")]
  LoadError(String),

  /// Error deserializing YAML manifest.
  #[error("Could not parse manifest {0} as YAML: {1}")]
  YamlError(String, String),

  /// Default was requested when none present.
  #[error("Connection '{0}' does not have a default but one was requested")]
  NoDefault(ConnectionDefinition),

  /// Error deserializing default value.
  #[error("Error deserializing default value for connection: {0}=>{1} - Error was: '{2}'")]
  DefaultsError(ConnectionTargetDefinition, ConnectionTargetDefinition, String),

  /// Error parsing or serializing Sender data.
  #[error("Error parsing or serializing Sender data: {0}")]
  InvalidSenderData(String),

  /// File path in manifest is invalid.
  #[error("Invalid file path: {0}")]
  BadPath(String),

  /// IP address in manifest is invalid.
  #[error("Invalid IP Address: {0}")]
  BadIpAddress(String),

  /// Invalid format of passed data. Check the error message for details.
  #[error("Invalid format: {0}")]
  Invalid(serde_json::Error),

  /// Invalid operation expression. Must be in the form component_name::operation_name.
  #[error("Invalid operation expression '{0}'. Must be in the form component_name::operation_name.")]
  InvalidOperationExpression(String),

  /// Parser error.
  #[error(transparent)]
  Parser(#[from] flow_expression_parser::Error),

  /// Miscellaneous error.
  #[error("General error : {0}")]
  Other(String),
}

impl From<std::io::Error> for ManifestError {
  fn from(e: std::io::Error) -> Self {
    Self::LoadError(e.to_string())
  }
}

#[derive(Error, Debug, Clone, Copy)]
/// Errors that can occur when trying to dereference a configuration name or id.
pub enum ReferenceError {
  /// The referenced item was not a component.
  #[error("Referenced item is not a component")]
  Component,
}
