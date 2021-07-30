use uuid::Uuid;

use crate::dev::prelude::*;

pub(crate) fn keypair_from_seed(seed: &str) -> Result<KeyPair, CommonError> {
  KeyPair::from_seed(seed).map_err(|_| CommonError::KeyPairFailed)
}

pub(crate) fn get_uuid() -> String {
  format!("{}", Uuid::new_v4())
}
