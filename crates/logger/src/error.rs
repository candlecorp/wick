use log::SetLoggerError;
use thiserror::Error;

#[derive(Error, Debug)]
/// The logger's Error enum
pub enum LoggerError {
  /// Upstream error
  #[error(transparent)]
  SetLoggerError(#[from] SetLoggerError),
}
