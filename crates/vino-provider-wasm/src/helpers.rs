use std::path::Path;

use crate::error::WasmProviderError;
pub use crate::wapc_module::WapcModule;

pub fn load_wasm_from_file(path: &Path) -> Result<WapcModule, WasmProviderError> {
  WapcModule::from_file(path)
}

pub async fn load_wasm_from_oci(
  path: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WapcModule, WasmProviderError> {
  let actor_bytes = oci_utils::fetch_oci_bytes(path, allow_latest, allowed_insecure).await?;
  Ok(WapcModule::from_slice(&actor_bytes)?)
}

pub async fn load_wasm(
  location: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WapcModule, WasmProviderError> {
  let path = Path::new(&location);
  if path.exists() {
    debug!("PRV:WASM:AS_FILE:{}", location);
    Ok(WapcModule::from_file(path)?)
  } else {
    debug!("PRV:WASM:AS_OCI:{}", location);
    load_wasm_from_oci(location, allow_latest, allowed_insecure).await
  }
}
