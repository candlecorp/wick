use anyhow::anyhow;
use thiserror::Error;

type WasmCloudError = Box<dyn std::error::Error + Sync + std::marker::Send>;

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
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<WasmCloudError> for VinoError {
    fn from(e: WasmCloudError) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl From<std::io::Error> for VinoError {
    fn from(e: std::io::Error) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

impl From<nkeys::error::Error> for VinoError {
    fn from(e: nkeys::error::Error) -> Self {
        VinoError::Other(anyhow!(e))
    }
}

// impl from<Box<ErrorImpl>> for VinoError {

// }
