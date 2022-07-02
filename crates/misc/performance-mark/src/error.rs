#[derive(Debug, Clone, Copy)]
/// This crate's Error object.
pub enum Error {
  /// `end()` called on a [crate::PerformancePeriod] before `start()`
  EndBeforeStart,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::EndBeforeStart => write!(f, "end() called before start()"),
    }
  }
}
