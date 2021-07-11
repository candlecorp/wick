use log::SetLoggerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoggerError {
  #[error(transparent)]
  SetLoggerError(#[from] SetLoggerError),
}
