use log::SetLoggerError;
use thiserror::Error;

#[derive(Error, Debug)]
/// The logger's Error enum.
pub enum LoggerError {
  /// Upstream error.
  #[error(transparent)]
  SetLoggerError(#[from] SetLoggerError),

  /// Invalid string passed as the log style.
  #[error("Could not parse log style. Log style should be auto | always | never")]
  StyleParse,
}
