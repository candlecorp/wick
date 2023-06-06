use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(missing_copy_implementations)]
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

  #[cfg(all(feature = "rng", not(target_family = "wasm")))]
  pub fn next(&self) -> Self {
    Self {
      seed: seeded_random::Random::from_seed(seeded_random::Seed::unsafe_new(self.seed)).gen(),
      timestamp: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap(),
    }
  }

  /// Create a new InherentData with the current time and a random seed.
  ///
  /// This is not "unsafe" in the Rust sense. It is unsafe because it should
  /// only be used if you are sure you know what you're doing. If you don't know why this is unsafe, don't use it.
  #[cfg(all(feature = "rng", feature = "std", not(target_family = "wasm")))]
  pub fn unsafe_default() -> Self {
    Self {
      seed: seeded_random::Random::new().gen(),
      timestamp: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap(),
    }
  }

  /// Clone the InherentData struct.
  ///
  /// This is not "unsafe" in the Rust sense. It is unsafe because it should
  /// only be used if you are sure you know what you're doing. If you don't know why this is unsafe, don't use it.
  pub fn unsafe_clone(&self) -> Self {
    Self {
      seed: self.seed,
      timestamp: self.timestamp,
    }
  }
}

#[cfg(test)]
mod tests {}
