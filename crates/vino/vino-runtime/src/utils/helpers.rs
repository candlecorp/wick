use rand::Rng;

use crate::dev::prelude::*;

pub(crate) fn keypair_from_seed(seed: &str) -> Result<KeyPair, CommonError> {
  KeyPair::from_seed(seed).map_err(|_| CommonError::KeyPairFailed)
}

#[must_use]
pub(crate) fn new_seed() -> u64 {
  let mut rng = rand::thread_rng();
  rng.gen()
}

pub(crate) trait OptionalState {
  type State: Send + Sync;

  fn get_state_option(&self) -> Option<&Self::State>;
  fn get_mut_state_option(&mut self) -> Option<&mut Self::State>;

  fn get_state(&self) -> Result<&Self::State, CommonError> {
    self.get_state_option().ok_or(CommonError::Uninitialized)
  }

  fn get_state_mut(&mut self) -> Result<&mut Self::State, CommonError> {
    self.get_mut_state_option().ok_or(CommonError::Uninitialized)
  }
}
