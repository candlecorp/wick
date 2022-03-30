use rand::Rng;

use crate::dev::prelude::*;

pub(crate) fn keypair_from_seed(seed: &str) -> Result<KeyPair, crate::Error> {
  KeyPair::from_seed(seed).map_err(|_| crate::Error::KeyPairFailed)
}

#[must_use]
pub(crate) fn new_seed() -> u64 {
  let mut rng = rand::thread_rng();
  rng.gen()
}
