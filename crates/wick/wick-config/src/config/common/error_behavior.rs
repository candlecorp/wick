#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
/// What to do when an error occurs.
#[serde(rename_all = "kebab-case")]
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

impl std::fmt::Display for ErrorBehavior {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Commit => write!(f, "commit"),
      Self::Rollback => write!(f, "rollback"),
      Self::Ignore => write!(f, "ignore"),
    }
  }
}
