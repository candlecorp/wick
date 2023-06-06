use std::sync::Arc;

use parking_lot::RwLock;
use rand::distributions::{Alphanumeric, Standard};
use rand::prelude::Distribution;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

use crate::Seed;

#[derive(Debug)]
#[must_use]
/// The main RNG data structure.
pub struct Random {
  rng: Arc<RwLock<ChaCha12Rng>>,
}

#[must_use]
#[cfg(feature = "std")]
pub(crate) fn new_seed() -> Seed {
  let mut rng = rand::thread_rng();
  Seed(rng.gen())
}

impl Random {
  /// Create a new RNG from a new, random seed.
  #[cfg(feature = "std")]
  pub fn new() -> Self {
    Self::from_seed(new_seed())
  }

  // Need to allow this lint so we can write an API that consumes the Seed even
  // though it's not technically necessary.
  #[allow(clippy::needless_pass_by_value)]
  /// Create a new [Random] RNG from a seed.
  pub fn from_seed(seed: Seed) -> Self {
    let rng = ChaCha12Rng::seed_from_u64(seed.0);
    Self {
      rng: Arc::new(RwLock::new(rng)),
    }
  }

  /// Generated a new seed from this RNG.
  pub fn seed(&self) -> Seed {
    Seed(self.gen())
  }

  #[must_use]
  /// Function that delegates to [rand::Rng::gen()]
  pub fn gen<T>(&self) -> T
  where
    Standard: Distribution<T>,
  {
    let mut rng = self.rng.write();
    rng.gen()
  }

  /// Utility function to generate a new [u32]
  pub fn u32(&self) -> u32 {
    self.gen()
  }

  /// Utility function to generate a new [i32]
  pub fn i32(&self) -> i32 {
    self.gen()
  }

  /// Utility function to generate a new [Vec] of bytes.
  pub fn bytes(&self, length: usize) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(length);
    let mut rng = self.rng.write();
    for _ in 0..length {
      bytes.push(rng.gen());
    }
    bytes
  }

  /// Utility function to generate a new [String]
  pub fn string(&self, length: usize) -> String {
    let mut string: String = String::with_capacity(length);
    let mut rng = self.rng.write();

    for _ in 0..length {
      string.push(rng.gen());
    }
    string
  }

  /// Utility function to generate a new [String] consisting only of numbers and letters.
  pub fn alphanumeric(&self, length: usize) -> String {
    let mut rng = self.rng.write();
    let chars: String = std::iter::repeat(())
      .map(|()| rng.sample(Alphanumeric))
      .map(char::from)
      .take(length)
      .collect();
    chars
  }

  /// Utility function to generate a new [uuid::Uuid]
  #[cfg(feature = "uuid")]
  pub fn uuid(&self) -> uuid::Uuid {
    let mut raw_bytes: [u8; 16] = [0; 16];
    let mut rng = self.rng.write();
    rng.fill(&mut raw_bytes);
    let bytes: uuid::Bytes = raw_bytes;
    let builder = uuid::Builder::from_bytes(bytes);
    builder.into_uuid()
  }

  /// Utility function that delegates to [rand::Rng::gen_range()]
  pub fn range(&self, min: u32, max: u32) -> u32 {
    let mut rng = self.rng.write();
    rng.gen_range(min..max)
  }
}

#[cfg(feature = "std")]
impl Default for Random {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bytes() {
    let rng = Random::from_seed(Seed(100000));
    let bytes1 = rng.bytes(10);
    let bytes2 = rng.bytes(10);
    assert_ne!(bytes1, bytes2);
    let rng = Random::from_seed(Seed(100000));
    let bytes2 = rng.bytes(10);
    assert_eq!(bytes1, bytes2);
  }
  #[test]
  fn string() {
    let rng = Random::from_seed(Seed(100000));
    let v1 = rng.string(10);
    let v2 = rng.string(10);
    assert_ne!(v1, v2);
    let rng = Random::from_seed(Seed(100000));
    let v2 = rng.string(10);
    assert_eq!(v1, v2);
  }

  #[test]
  fn alphanum() {
    let rng = Random::from_seed(Seed(100000));
    let v1 = rng.alphanumeric(10);
    let v2 = rng.alphanumeric(10);
    assert_ne!(v1, v2);
    let rng = Random::from_seed(Seed(100000));
    let v2 = rng.alphanumeric(10);
    assert_eq!(v1, v2);
  }

  #[test]
  #[cfg(feature = "uuid")]
  fn uuid() {
    let rng = Random::from_seed(Seed(100000));
    let v1 = rng.uuid();
    let v2 = rng.uuid();
    assert_ne!(v1, v2);
    let rng = Random::from_seed(Seed(100000));
    let v2 = rng.uuid();
    assert_eq!(v1, v2);
  }
}
