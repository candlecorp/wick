#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum TimeError {
  #[error("bad schedule, unable to create schedule from cron expression '{0}'")]
  BadSchedule(String, #[source] cron::error::Error),

  #[error("error in configuration: {0}")]
  Config(Box<wick_config::Error>),
}

impl From<TimeError> for wick_trigger::Error {
  fn from(value: TimeError) -> Self {
    wick_trigger::Error::new_context("time", wick_trigger::ErrorKind::Trigger(Box::new(value)))
  }
}

impl From<wick_config::Error> for TimeError {
  fn from(value: wick_config::Error) -> Self {
    TimeError::Config(Box::new(value))
  }
}
