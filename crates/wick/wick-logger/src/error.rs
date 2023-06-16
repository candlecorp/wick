use thiserror::Error;

#[derive(Error, Debug, Clone)]
/// The logger's Error enum.
pub enum LoggerError {
  /// Invalid string passed as the log style.
  #[error("Could not parse log style. Log style should be auto | always | never")]
  StyleParse,

  /// IO Error.
  #[error("I/O error: {0}")]
  IOError(String),

  /// Invalid string passed as the log style.
  #[error("Could not create logfile {0}")]
  NoLogfile(String),

  /// Error resolving platform-specific configuration.
  #[error("Error resolving platform-specific configuration: {0}")]
  Platform(#[from] wick_xdg::Error),

  /// General initialization error.
  #[error("Could not initialize logger: {0}")]
  InitFailed(String),
}

impl From<std::io::Error> for LoggerError {
  fn from(e: std::io::Error) -> Self {
    LoggerError::IOError(e.to_string())
  }
}
impl From<tracing::dispatcher::SetGlobalDefaultError> for LoggerError {
  fn from(e: tracing::dispatcher::SetGlobalDefaultError) -> Self {
    LoggerError::InitFailed(e.to_string())
  }
}
