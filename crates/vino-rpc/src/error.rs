use thiserror::Error;

type BoxedError = Box<dyn std::error::Error + Sync + std::marker::Send>;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Deserialization error {0}")]
    RpcMessageError(&'static str),
    #[error("Client is shutting down, streams are closing")]
    ShuttingDown,
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
    #[error("Error {0}")]
    Other(String),
    #[error(transparent)]
    VinoError(#[from] vino_runtime::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    UpstreamError(#[from] BoxedError),
}

impl From<&str> for RpcError {
    fn from(s: &str) -> Self {
        Self::Other(s.to_string())
    }
}
