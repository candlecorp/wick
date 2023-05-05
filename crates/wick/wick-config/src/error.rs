use thiserror::Error;

use crate::config::{self};

// type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

/// Wick Manifest's Errors.
#[derive(Error, Debug)]
pub enum ManifestError {
  /// Invalid version found in the parsed manifest.
  #[error("Invalid Manifest Version '{0}'")]
  VersionError(String),

  /// Error related to asset references.
  #[error(transparent)]
  AssetReference(#[from] wick_asset_reference::Error),

  /// Error related to fetching asset references.
  #[error(transparent)]
  AssetContainer(#[from] asset_container::Error),

  /// Attempted to retrieve the types in a manifest before they've been fetched.
  #[error("Attempted to retrieve the types in a manifest before they've been fetched")]
  TypesNotFetched,

  /// Attempted to import a type that was not found in the manifest.
  #[error("Attempted to import a type that was not found in the manifest: {0}")]
  TypeNotFound(String),

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
  #[error("Could not parse manifest {} as YAML: {1} at line {}, column {}", .0.as_ref().map_or("<raw>".to_owned(), |v|v.display().to_string()), .2.as_ref().map_or("unknown".to_owned(),|l|l.line().to_string()), .2.as_ref().map_or("unknown".to_owned(),|l|l.column().to_string()))]
  YamlError(Option<std::path::PathBuf>, String, Option<serde_yaml::Location>),

  /// Error parsing or serializing Sender data.
  #[error("Error parsing or serializing Sender data: {0}")]
  InvalidSenderData(String),

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

  /// Type Parser error.
  #[error(transparent)]
  TypeParser(#[from] wick_interface_types::ParserError),

  /// Error parsing YAML as a string.
  #[error("Error parsing YAML as a string")]
  Utf8,

  /// Invalid authority format
  #[error("Invalid authority: {0}")]
  InvalidUrl(String),
}

#[derive(Error, Debug, Clone, Copy)]
/// Errors that can occur when trying to dereference a configuration name or id.
pub enum ReferenceError {
  /// The referenced item was not a component.
  #[error("Referenced item is not a component")]
  Component,
  /// The referenced item was not a resource.
  #[error("Referenced item is not a resource")]
  Resource,
}
