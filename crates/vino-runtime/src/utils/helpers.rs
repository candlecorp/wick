use rand::Rng;
use uuid::Uuid;

use crate::dev::prelude::*;

pub(crate) fn keypair_from_seed(seed: &str) -> Result<KeyPair, CommonError> {
  KeyPair::from_seed(seed).map_err(|_| CommonError::KeyPairFailed)
}

pub(crate) fn get_uuid() -> String {
  format!("{}", Uuid::new_v4())
}

#[must_use]
pub(crate) fn new_seed() -> u64 {
  let mut rng = rand::thread_rng();
  rng.gen()
}
