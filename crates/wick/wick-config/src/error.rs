use std::path::PathBuf;

use thiserror::Error;

// type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

/// Wick Manifest's Errors.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ManifestError {
  /// Invalid version found in the parsed manifest.
  #[error("Invalid Manifest Version '{0}'")]
  VersionError(String),

  /// Error related to asset references.
  #[cfg(feature = "config")]
  #[error(transparent)]
  AssetReference(#[from] wick_asset_reference::Error),

  /// Error related to fetching asset references.
  #[cfg(feature = "config")]
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
  #[cfg(feature = "config")]
  #[cfg_attr(
    feature = "config",
    error("Expected a {0} configuration but got a {1} configuration")
  )]
  UnexpectedConfigurationKind(crate::config::ConfigurationKind, crate::config::ConfigurationKind),

  /// Thrown when a specific type of component was expected but a different type was found.
  #[cfg(feature = "config")]
  #[cfg_attr(feature = "config", error("Expected a {0} component but got a {1} component"))]
  UnexpectedComponentType(crate::config::ComponentKind, crate::config::ComponentKind),

  /// Error deserializing YAML manifest.
  #[error("Could not parse manifest {} as YAML: {1} at line {}, column {}", .0.as_ref().map_or("<raw>".to_owned(), |v|v.display().to_string()), .2.as_ref().map_or("unknown".to_owned(),|l|l.line().to_string()), .2.as_ref().map_or("unknown".to_owned(),|l|l.column().to_string()))]
  YamlError(Option<PathBuf>, String, Option<serde_yaml::Location>),

  /// IP address in manifest is invalid.
  #[error("Invalid IP Address: {0}")]
  BadIpAddress(String),

  /// Invalid regular expression.
  #[error("Invalid regular expression: {0}")]
  InvalidRegex(String),

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

  /// Error converting an enum to a specific variant.
  #[error("could not convert a {0} into a {1}")]
  VariantError(String, String),

  /// Error parsing YAML as a string.
  #[error("Error parsing YAML as a string")]
  Utf8,

  /// Invalid authority format
  #[error("Invalid authority: {0}")]
  InvalidUrl(String),

  /// Identifier not found.
  #[error("id '{id}' undefined, IDs in scope are: {}", .ids.join(", "))]
  IdNotFound {
    /// The lookup id.
    id: String,
    /// The ids that exist in the lookup scope
    ids: Vec<String>,
  },

  /// Attempted to use configuration before it was renderable.
  #[error("Could not render configuration template: {0}")]
  ConfigurationTemplate(String),

  /// Passed [wick_packet::RuntimeConfig] is invalid for the configuration required by this component.
  #[cfg(feature = "config")]
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
  #[cfg(feature = "config")]
  #[cfg_attr(feature = "config", error(transparent))]
  Reference(#[from] ReferenceError),

  /// Error building a configuration
  #[cfg(feature = "config")]
  #[error(transparent)]
  Builder(#[from] BuilderError),

  /// Error converting configured Packet flags.
  #[error("Error converting configured Packet flags, use the singular version instead")]
  InvalidPacketFlags,
}

#[cfg(feature = "config")]
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

  /// The resource was not the requested type.
  #[error("Expected a resource of type {expected}, found type {actual}")]
  ResourceType {
    /// The expected resource type.
    expected: crate::config::resources::ResourceKind,
    /// The actual resource type.
    actual: crate::config::resources::ResourceKind,
  },
}

#[cfg(feature = "config")]
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

#[cfg(feature = "config")]
impl From<String> for BuilderError {
  fn from(s: String) -> Self {
    Self::ValidationError(s)
  }
}

#[cfg(feature = "config")]
impl From<derive_builder::UninitializedFieldError> for BuilderError {
  fn from(value: derive_builder::UninitializedFieldError) -> Self {
    Self::UninitializedField(value.field_name())
  }
}
