use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
/// Data inherent to an invocation. Meant to be supplied by a runtime, not a user.
#[must_use]
pub struct InherentData {
  /// The seed to associate with an invocation.
  pub seed: u64,
  /// The timestamp to associate with an invocation.
  pub timestamp: u64,
}

impl InherentData {
  /// Constructor for [InherentData]
  pub fn new(seed: u64, timestamp: u64) -> Self {
    Self { seed, timestamp }
  }
}

#[cfg(test)]
mod tests {}
