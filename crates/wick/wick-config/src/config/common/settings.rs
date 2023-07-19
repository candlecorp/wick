#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
#[allow(missing_copy_implementations)]
/// Settings related to execution behavior.
pub struct ExecutionSettings {
  /// The timeout for the execution.
  pub timeout: Option<Duration>,
}

impl ExecutionSettings {
  /// Create a new settings object.
  #[must_use]
  pub fn new(timeout: Option<Duration>) -> Self {
    Self { timeout }
  }

  /// Create a new settings object with a timeout from milliseconds.
  #[must_use]
  pub fn from_timeout_millis(millis: u64) -> Self {
    Self {
      timeout: Some(Duration::from_millis(millis)),
    }
  }

  /// Get the timeout duration as milliseconds if set.
  #[must_use]
  pub fn timeout_millis(&self) -> Option<u64> {
    self.timeout.map(|d| d.as_millis() as _)
  }
}
