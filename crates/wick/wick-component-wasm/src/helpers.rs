use std::path::Path;

use wick_loader_utils::{get_bytes, get_bytes_from_oci};

use crate::error::WasmCollectionError;
pub use crate::wasm_module::WickWasmModule;

pub async fn load_wasm_from_file(path: &Path) -> Result<WickWasmModule, WasmCollectionError> {
  WickWasmModule::from_file(path).await
}

pub async fn load_wasm_from_oci(
  path: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WickWasmModule, WasmCollectionError> {
  let actor_bytes = get_bytes_from_oci(path, allow_latest, allowed_insecure).await?;
  WickWasmModule::from_slice(&actor_bytes)
}

pub async fn load_wasm(
  location: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WickWasmModule, WasmCollectionError> {
  let bytes = get_bytes(location, allow_latest, allowed_insecure).await?;
  WickWasmModule::from_slice(&bytes)
}
