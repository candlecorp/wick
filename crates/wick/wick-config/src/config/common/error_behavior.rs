#[derive(Debug, Clone, Copy, PartialEq)]
/// What to do when an error occurs.
pub enum ErrorBehavior {
  /// The operation will commit what has succeeded.
  Commit = 0,
  /// The operation will rollback changes.
  Rollback = 1,
  /// Errors will be ignored.
  Ignore = 2,
}

impl Default for ErrorBehavior {
  fn default() -> Self {
    Self::Ignore
  }
}
