use std::sync::Arc;

use parking_lot::RwLock;
use rand::distributions::Alphanumeric;
use rand::{
  Rng,
  SeedableRng,
};
use rand_chacha::ChaCha12Rng;

#[derive(Debug, Clone)]
pub struct Seed {}
#[derive(Debug, Clone)]
pub struct Random {
  rng: Arc<RwLock<ChaCha12Rng>>,
}

#[must_use]
pub(crate) fn new_seed() -> u64 {
  let mut rng = rand::thread_rng();
  rng.gen()
}

impl Random {
  pub fn new() -> Self {
    Self::from_seed(new_seed())
  }

  pub fn from_seed(seed: u64) -> Self {
    let rng = ChaCha12Rng::seed_from_u64(seed);
    Self {
      rng: Arc::new(RwLock::new(rng)),
    }
  }

  pub fn get_bytes(&self, length: usize) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(length);
    let mut rng = self.rng.write();
    for _ in 0..length {
      bytes.push(rng.gen());
    }
    bytes
  }

  pub fn get_string(&self, length: usize) -> String {
    let mut string: String = String::with_capacity(length);
    let mut rng = self.rng.write();

    for _ in 0..length {
      string.push(rng.gen());
    }
    string
  }

  pub fn get_alphanumeric(&self, length: usize) -> String {
    let mut rng = self.rng.write();
    let chars: String = std::iter::repeat(())
      .map(|()| rng.sample(Alphanumeric))
      .map(char::from)
      .take(length)
      .collect();
    chars
  }

  pub fn get_uuid(&self) -> String {
    let mut raw_bytes: [u8; 16] = [0; 16];
    let mut rng = self.rng.write();
    rng.fill(&mut raw_bytes);
    let bytes: uuid::Bytes = raw_bytes;
    let mut builder = uuid::Builder::from_bytes(bytes);
    builder.build().to_hyphenated().to_string()
  }
}

impl Default for Random {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  static SEED: u64 = 10000;

  #[test]
  fn bytes() {
    let rng = Random::from_seed(SEED);
    let bytes1 = rng.get_bytes(10);
    let bytes2 = rng.get_bytes(10);
    assert_ne!(bytes1, bytes2);
    let rng = Random::from_seed(SEED);
    let bytes2 = rng.get_bytes(10);
    assert_eq!(bytes1, bytes2);
  }
  #[test]
  fn string() {
    let rng = Random::from_seed(SEED);
    let v1 = rng.get_string(10);
    let v2 = rng.get_string(10);
    assert_ne!(v1, v2);
    let rng = Random::from_seed(SEED);
    let v2 = rng.get_string(10);
    assert_eq!(v1, v2);
  }

  #[test]
  fn alphanum() {
    let rng = Random::from_seed(SEED);
    let v1 = rng.get_alphanumeric(10);
    let v2 = rng.get_alphanumeric(10);
    assert_ne!(v1, v2);
    let rng = Random::from_seed(SEED);
    let v2 = rng.get_alphanumeric(10);
    assert_eq!(v1, v2);
  }

  #[test]
  fn uuid() {
    let rng = Random::from_seed(SEED);
    let v1 = rng.get_uuid();
    let v2 = rng.get_uuid();
    assert_ne!(v1, v2);
    let rng = Random::from_seed(SEED);
    let v2 = rng.get_uuid();
    assert_eq!(v1, v2);
  }
}
