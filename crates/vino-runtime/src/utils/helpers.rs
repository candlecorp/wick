use uuid::Uuid;
pub use vino_provider_wasm::wapc_module::WapcModule;

use crate::dev::prelude::*;
pub use crate::providers::network_provider::Provider as NetworkProvider;

pub(crate) fn keypair_from_seed(seed: &str) -> Result<KeyPair, CommonError> {
  KeyPair::from_seed(seed).map_err(|_| CommonError::KeyPairFailed)
}

pub(crate) fn get_uuid() -> String {
  format!("{}", Uuid::new_v4())
}
