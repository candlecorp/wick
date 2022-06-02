use seeded_random::{Random, Seed};

use crate::dev::prelude::*;

pub(crate) fn keypair_from_seed(seed: &str) -> Result<KeyPair, crate::Error> {
  KeyPair::from_seed(seed).map_err(|_| crate::Error::KeyPairFailed)
}

#[must_use]
pub(crate) fn new_seed() -> Seed {
  Random::new().seed()
}
