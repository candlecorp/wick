use crate::error::ErrorContext;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum TimeError {
  #[error("bad schedule, unable to create schedule from cron expression '{0}'")]
  BadSchedule(String, #[source] cron::error::Error),
}

impl From<TimeError> for crate::error::Error {
  fn from(value: TimeError) -> Self {
    crate::error::Error::new_context(ErrorContext::Time, crate::error::ErrorKind::Time(Box::new(value)))
  }
}
