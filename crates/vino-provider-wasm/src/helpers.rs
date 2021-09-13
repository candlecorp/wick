use std::path::Path;

use vino_loader::{
  get_bytes,
  get_bytes_from_oci,
};

use crate::error::WasmProviderError;
pub use crate::wapc_module::WapcModule;

pub async fn load_wasm_from_file(path: &Path) -> Result<WapcModule, WasmProviderError> {
  WapcModule::from_file(path).await
}

pub async fn load_wasm_from_oci(
  path: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WapcModule, WasmProviderError> {
  let actor_bytes = get_bytes_from_oci(path, allow_latest, allowed_insecure).await?;
  Ok(WapcModule::from_slice(&actor_bytes)?)
}

pub async fn load_wasm(
  location: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WapcModule, WasmProviderError> {
  let bytes = get_bytes(location, allow_latest, allowed_insecure).await?;
  WapcModule::from_slice(&bytes)
}
