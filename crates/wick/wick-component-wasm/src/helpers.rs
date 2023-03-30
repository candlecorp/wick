use std::path::Path;

use wick_config::config::{FetchOptions, LocationReference};

use crate::error::WasmCollectionError;
pub use crate::wasm_module::WickWasmModule;

pub async fn load_wasm_from_file(path: &Path) -> Result<WickWasmModule, WasmCollectionError> {
  WickWasmModule::from_file(path).await
}

pub async fn load_wasm(
  location: &LocationReference,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WickWasmModule, WasmCollectionError> {
  let options = FetchOptions::new()
    .allow_latest(allow_latest)
    .allow_insecure(allowed_insecure);
  WickWasmModule::from_slice(&location.bytes(&options).await?)
}
