use thiserror::Error;

#[derive(Error, Debug)]
pub enum VinoError {
    #[error("invalid configuration")]
    ConfigurationError,
}
