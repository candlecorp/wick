use actix::prelude::SendError;
use anyhow::anyhow;
use thiserror::Error;

type BoxedErrorSyncSend = Box<dyn std::error::Error + Sync + std::marker::Send>;
// type BoxedError = Box<dyn std::error::Error>;

#[derive(Error, Debug)]
pub enum VinoError {
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
    #[error("Failed to deserialize configuration {0}")]
    ConfigurationDeserialization(String),
    #[error("Failed to serialize payload {0}")]
    SerializationError(rmp_serde::encode::Error),
    #[error("Failed to deserialize payload {0}")]
    DeserializationError(rmp_serde::decode::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<BoxedErrorSyncSend> for VinoError {
    fn from(e: BoxedErrorSyncSend) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl From<serde_yaml::Error> for VinoError {
    fn from(e: serde_yaml::Error) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl From<wascap::Error> for VinoError {
    fn from(e: wascap::Error) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl From<bcrypt::BcryptError> for VinoError {
    fn from(e: bcrypt::BcryptError) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl From<actix::MailboxError> for VinoError {
    fn from(e: actix::MailboxError) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl From<std::io::Error> for VinoError {
    fn from(e: std::io::Error) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl From<String> for VinoError {
    fn from(e: String) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl From<&'static str> for VinoError {
    fn from(e: &'static str) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl From<nkeys::error::Error> for VinoError {
    fn from(e: nkeys::error::Error) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl<M> From<SendError<M>> for VinoError {
    fn from(e: SendError<M>) -> Self {
        VinoError::Other(anyhow!(e.to_string()))
    }
}
