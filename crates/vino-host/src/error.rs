use thiserror::Error;

type BoxedErrorSyncSend = Box<dyn std::error::Error + Sync + Send>;
// type BoxedError = Box<dyn std::error::Error>;

#[derive(Error, Debug)]
pub enum VinoHostError {
  #[error("invalid configuration")]
  ConfigurationError,
  #[error("File not found {0}")]
  FileNotFound(String),
  #[error("Configuration disallows fetching artifacts with the :latest tag ({0})")]
  LatestDisallowed(String),
  #[error("Could not fetch '{0}': {1}")]
  OciFetchFailure(String, String),
  #[error("Could not start host: {0}")]
  HostStartFailure(String),
  #[error(transparent)]
  VinoError(#[from] vino_runtime::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
  #[error(transparent)]
  ManifestError(#[from] vino_manifest::Error),
  #[error("Invalid host state for operation: {0}")]
  InvalidHostState(String),
  #[error("Failed to deserialize configuration {0}")]
  ConfigurationDeserialization(String),
  #[error(transparent)]
  YamlError(#[from] serde_yaml::Error),
  #[error(transparent)]
  HoconError(#[from] hocon::Error),
  #[error(transparent)]
  ActixMailboxError(#[from] actix::MailboxError),
  #[error(transparent)]
  IOError(#[from] std::io::Error),
  #[error(transparent)]
  KeyPairError(#[from] nkeys::error::Error),
  #[error("General error : {0}")]
  Other(String),
}

impl From<BoxedErrorSyncSend> for VinoHostError {
  fn from(e: BoxedErrorSyncSend) -> Self {
    VinoHostError::Other(e.to_string())
  }
}

impl From<String> for VinoHostError {
  fn from(e: String) -> Self {
    VinoHostError::Other(e)
  }
}

impl From<&'static str> for VinoHostError {
  fn from(e: &'static str) -> Self {
    VinoHostError::Other(e.to_string())
  }
}
