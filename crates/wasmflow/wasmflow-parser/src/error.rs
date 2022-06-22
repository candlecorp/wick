use thiserror::Error;

// type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

/// Wasmflow Manifest's Errors.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParserError {
  /// Invalid version found in the parsed manifest.
  #[error("Invalid Manifest Version '{0}'")]
  VersionError(String),

  /// No version found in the parsed manifest.
  #[error("Manifest needs a version")]
  NoVersion,

  /// Manifest not found at the specified path.
  #[error("File not found {0}")]
  FileNotFound(String),

  /// Could not load file.
  #[error("Could not read file {0}")]
  LoadError(String),

  /// Component id is not a fully qualified name with a namespace.
  #[error("Component id '{0}' is not a fully qualified name with a namespace")]
  ComponentIdError(String),

  /// General deserialization error.
  #[error("Failed to deserialize configuration {0}")]
  ConfigurationDeserialization(String),

  /// Error deserializing YAML manifest.
  #[error("Could not parse manifest as YAML: {0}")]
  YamlError(String),

  /// Default was requested when none present.
  #[error("Invalid connection target syntax: '{0}'")]
  ConnectionTargetSyntax(String),

  /// Default was requested when none present.
  #[error("Invalid connection definition syntax: '{0}'")]
  ConnectionDefinitionSyntax(String),

  /// Ambiguous reference in connection shorthand.
  #[error("No suitable default found for port in : {0}")]
  NoDefaultPort(String),

  /// Ambiguous port in connection shorthand.
  #[error("No suitable default found for reference in : {0}")]
  NoDefaultReference(String),

  /// Error parsing or serializing Sender data.
  #[error("Error parsing or serializing Sender data: {0}")]
  InvalidSenderData(String),

  /// Error attempting to get details of a target that doesn't exist.
  #[error("Attempted to grab data from a target that doesn't exist")]
  NoTarget,

  /// File path in manifest is invalid.
  #[error("Invalid file path: {0}")]
  BadPath(String),

  /// IP address in manifest is invalid.
  #[error("Invalid IP Address: {0}")]
  BadIpAddress(String),

  /// Miscellaneous error.
  #[error("General error : {0}")]
  Other(String),
}

impl From<std::io::Error> for ParserError {
  fn from(e: std::io::Error) -> Self {
    Self::LoadError(e.to_string())
  }
}
