use wick_config::config::{AssetReference, FetchOptions};

use crate::error::WasmComponentError;
pub use crate::wasm_module::WickWasmModule;

pub async fn fetch_wasm(
  location: &AssetReference,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WickWasmModule, WasmComponentError> {
  let options = FetchOptions::new()
    .allow_latest(allow_latest)
    .allow_insecure(allowed_insecure);
  WickWasmModule::from_slice(&location.bytes(&options).await?)
}
