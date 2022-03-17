#[derive(Debug, Clone, Copy)]
pub enum Error {
  EndBeforeStart,
  PeriodOpen,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::EndBeforeStart => write!(f, "end() called before start()"),
      Error::PeriodOpen => write!(
        f,
        "PerformancePeriod never closed. Call end() before querying the period."
      ),
    }
  }
}
