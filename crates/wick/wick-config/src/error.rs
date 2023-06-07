use std::path::PathBuf;

use derive_builder::UninitializedFieldError;
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

  /// Error rendering liquid JSON configuration.
  #[error(transparent)]
  LiquidConfig(#[from] liquid_json::Error),

  /// Attempted to retrieve the types in a manifest before they've been fetched.
  #[error("Attempted to retrieve the types in a manifest before they've been fetched")]
  TypesNotFetched,

  /// Attempted to import a type that was not found in the manifest.
  #[error("Attempted to import a type that was not found in the manifest: {0}")]
  TypeNotFound(String),

  /// No format version or kind found in the parsed manifest.
  #[error("Manifest {} needs a format version (v0) or kind (v1+)", .0.as_ref().map_or("<raw>".to_owned(), |v|v.display().to_string()))]
  NoFormat(Option<PathBuf>),

  /// Manifest not found at the specified path.
  #[error("File not found {0}")]
  FileNotFound(String),

  /// Could not load file.
  #[error("Could not load file {0}")]
  LoadError(String),

  /// Thrown when a specific type of configuration was expected but a different type was found.
  #[error("Expected a {0} configuration but got a {1} configuration")]
  UnexpectedConfigurationKind(config::ConfigurationKind, config::ConfigurationKind),

  /// Thrown when a specific type of component was expected but a different type was found.
  #[error("Expected a {0} component but got a {1} component")]
  UnexpectedComponentType(config::ComponentKind, config::ComponentKind),

  /// Error deserializing YAML manifest.
  #[error("Could not parse manifest {} as YAML: {1} at line {}, column {}", .0.as_ref().map_or("<raw>".to_owned(), |v|v.display().to_string()), .2.as_ref().map_or("unknown".to_owned(),|l|l.line().to_string()), .2.as_ref().map_or("unknown".to_owned(),|l|l.column().to_string()))]
  YamlError(Option<PathBuf>, String, Option<serde_yaml::Location>),

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

  /// Attempted to use configuration before it was renderable.
  #[error("Could not render configuration template: {0}")]
  ConfigurationTemplate(String),

  /// Passed [RuntimeConfig] is invalid for the configuration required by this component.
  #[error(transparent)]
  ConfigurationInvalid(#[from] wick_packet::Error),

  /// Attempted to serialize a complex [liquid_json::LiquidJson] template into a string.
  #[error("Invalid template: could not turn an object or an array into a string")]
  TemplateStructure,

  /// Attempted to create a template render context from a non-object value.
  #[error("Invalid template context, context must be an object")]
  ContextBase,

  /// Attempted to use configuration before it was rendered.
  #[error("Attempted to use configuration '{0}' before it was rendered")]
  UnrenderedConfiguration(String),

  /// Error resolving reference.
  #[error(transparent)]
  Reference(ReferenceError),

  /// Error building a configuration
  #[error(transparent)]
  Builder(#[from] BuilderError),
}

#[derive(Error, Debug, Clone, Copy)]
/// Errors that can occur when trying to dereference a configuration name or id.
pub enum ReferenceError {
  /// The referenced item was not a component.
  #[error("Referenced item is not a component")]
  Component,

  /// The referenced item was not an imported types configuration.
  #[error("Referenced item is not an imported types configuration")]
  Types,

  /// The referenced item was not a resource.
  #[error("Referenced item is not a resource")]
  Resource,
}
/// Errors generated when building a configuration.
#[derive(Error, Debug)]
pub enum BuilderError {
  /// Uninitialized field
  #[error("Uninitialized field: {0}")]
  UninitializedField(&'static str),
  /// Invalid builder configuration
  #[error("Invalid builder configuration: {0}")]
  ValidationError(String),
}
impl From<String> for BuilderError {
  fn from(s: String) -> Self {
    Self::ValidationError(s)
  }
}
impl From<UninitializedFieldError> for BuilderError {
  fn from(value: UninitializedFieldError) -> Self {
    Self::UninitializedField(value.field_name())
  }
}
