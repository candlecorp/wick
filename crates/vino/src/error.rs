use thiserror::Error;
use vino_manifest::error::ManifestError;

// type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

#[derive(Error, Debug)]
pub enum VinoError {
  #[error("invalid configuration")]
  ConfigurationError,
  #[error("Manifest load failed: {0}")]
  ManifestLoadFail(String),
  #[error("File not found {0}")]
  FileNotFound(String),
  #[error("Configuration disallows fetching artifacts with the :latest tag ({0})")]
  LatestDisallowed(String),
  #[error("Could not fetch '{0}': {1}")]
  OciFetchFailure(String, String),
  #[error("Could not start host: {0}")]
  HostStartFailure(String),
  #[error("Failed to deserialize configuration {0}")]
  ConfigurationDeserialization(String),
  #[error(transparent)]
  CliError(#[from] vino_provider_cli::Error),
  #[error(transparent)]
  VinoHostError(#[from] vino_host::Error),
  #[error(transparent)]
  VinoRuntimeError(#[from] vino_runtime::Error),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  TransportError(#[from] vino_transport::error::TransportError),
  #[error(transparent)]
  SerdeJsonError(#[from] serde_json::Error),
  #[error(transparent)]
  LoggerError(#[from] logger::error::LoggerError),
  #[error("General error : {0}")]
  Other(String),
  #[error("Manifest failed to load : {0}")]
  ManifestLoadFailure(String),
}

impl From<ManifestError> for VinoError {
  fn from(e: ManifestError) -> Self {
    VinoError::ManifestLoadFailure(e.to_string())
  }
}
