use std::path::PathBuf;

use thiserror::Error;

use crate::config::{self};

// type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

/// Wick Manifest's Errors.
#[derive(Error, Debug)]
pub enum ManifestError {
  /// Invalid version found in the parsed manifest.
  #[error("Invalid Manifest Version '{0}'")]
  VersionError(String),

  /// Could not create package cache.
  #[error("Could not create package cache at '{path}': {error}")]
  PackageCacheError { path: PathBuf, error: std::io::Error },

  /// No data stored in the current location reference.
  #[error("No data available in the location reference at {0}")]
  NoData(String),

  /// No format version or kind found in the parsed manifest.
  #[error("Manifest needs a format version (v0) or kind (v1+)")]
  NoFormat,

  /// Manifest not found at the specified path.
  #[error("File not found {0}")]
  FileNotFound(String),

  /// Could not load file.
  #[error("Could not read file {0}: {1}")]
  LoadError(String, String),

  /// Thrown when a specific type of configuration was expected but a different type was found.
  #[error("Expected a {0} configuration but got a {1} configuration")]
  UnexpectedConfigurationKind(config::ConfigurationKind, config::ConfigurationKind),

  /// Thrown when a specific type of component was expected but a different type was found.
  #[error("Expected a {0} component but got a {1} component")]
  UnexpectedComponentType(config::ComponentKind, config::ComponentKind),

  /// Error deserializing YAML manifest.
  #[error("Could not parse manifest {0} as YAML: {1}")]
  YamlError(String, String),

  /// Default was requested when none present.
  #[error("Connection '{0}' does not have a default but one was requested")]
  NoDefault(config::ConnectionDefinition),

  /// Error deserializing default value.
  #[error("Error deserializing default value for connection: {0}=>{1} - Error was: '{2}'")]
  DefaultsError(
    config::ConnectionTargetDefinition,
    config::ConnectionTargetDefinition,
    String,
  ),

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

#[derive(Error, Debug, Clone, Copy)]
/// Errors that can occur when trying to dereference a configuration name or id.
pub enum ReferenceError {
  /// The referenced item was not a component.
  #[error("Referenced item is not a component")]
  Component,
}
