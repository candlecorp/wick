use thiserror::Error;

// type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("Invalid configuration")]
    ConfigurationError,
    #[error("Invalid Manifest Version '{0}'")]
    VersionError(String),
    #[error("File not found {0}")]
    FileNotFound(String),
    #[error("Failed to deserialize configuration {0}")]
    ConfigurationDeserialization(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    HoconError(#[from] hocon::Error),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error("General error : {0}")]
    Other(String),
}
