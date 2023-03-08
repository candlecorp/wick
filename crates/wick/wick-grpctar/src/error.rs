use thiserror::Error;

#[derive(Error, Debug)]
/// Crate error.
pub enum ParError {
  /// IO error for the local cache.
  #[error(transparent)]
  IOError(#[from] std::io::Error),

  /// JSON Parse Error
  #[error(transparent)]
  JsonParseFailed(#[from] serde_json::Error),

  /// YAML Parse Error
  #[error(transparent)]
  YamlParseFailed(#[from] serde_yaml::Error),

  /// Wascap Error
  #[error(transparent)]
  WasCap(#[from] wick_wascap::Error),

  /// Collection archive invalid
  #[error("Collection archive invalid")]
  ArchiveInvalid(String),

  /// Collection archive is missing the JWT
  #[error("Collection archive is missing the JWT")]
  MissingJwt,

  /// Collection archive is missing the interface schema
  #[error("Collection archive is missing the interface schema")]
  MissingInterface,

  /// Collection archive is missing the binary
  #[error("Collection archive is missing the binary")]
  MissingBinary,
}
