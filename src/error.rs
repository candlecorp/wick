use anyhow::anyhow;
use thiserror::Error;

type WasmCloudError = Box<dyn std::error::Error + Sync + std::marker::Send>;

#[derive(Error, Debug)]
pub enum VinoError {
    #[error("invalid configuration")]
    ConfigurationError,
    #[error("file not found {0}")]
    FileNotFound(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<WasmCloudError> for VinoError {
    fn from(e: WasmCloudError) -> Self {
        VinoError::Other(anyhow!(e))
    }
}
