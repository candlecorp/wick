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

// impl From<wascap::Error> for VinoError {
//     fn from(e: wascap::Error) -> Self {
//         VinoError::Other(anyhow!(e))
//     }
// }

// impl From<bcrypt::BcryptError> for VinoError {
//     fn from(e: bcrypt::BcryptError) -> Self {
//         VinoError::Other(anyhow!(e))
//     }
// }

// impl From<actix::MailboxError> for VinoHostError {
//     fn from(e: actix::MailboxError) -> Self {
//         VinoHostError::Other(anyhow!(e))
//     }
// }

// impl From<std::io::Error> for VinoHostError {
//     fn from(e: std::io::Error) -> Self {
//         VinoHostError::Other(anyhow!(e))
//     }
// }

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

// impl From<nkeys::error::Error> for VinoError {
//     fn from(e: nkeys::error::Error) -> Self {
//         VinoError::Other(anyhow!(e))
//     }
// }

// impl<M> From<SendError<M>> for VinoHostError {
//     fn from(e: SendError<M>) -> Self {
//         VinoHostError::Other(anyhow!(e.to_string()))
//     }
// }
