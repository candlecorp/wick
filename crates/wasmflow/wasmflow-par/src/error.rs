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
  WasCap(#[from] wasmflow_wascap::Error),

  /// Provider archive invalid
  #[error("Provider archive invalid")]
  ArchiveInvalid(String),

  /// Provider archive is missing the JWT
  #[error("Provider archive is missing the JWT")]
  MissingJwt,

  /// Provider archive is missing the interface schema
  #[error("Provider archive is missing the interface schema")]
  MissingInterface,

  /// Provider archive is missing the binary
  #[error("Provider archive is missing the binary")]
  MissingBinary,
}
