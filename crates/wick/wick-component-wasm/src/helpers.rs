use wick_config::config::{AssetReference, FetchOptions};

use crate::error::WasmComponentError;
pub use crate::wasm_module::WickWasmModule;

pub async fn fetch_wasm(
  location: &AssetReference,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WickWasmModule, WasmComponentError> {
  let mut options = FetchOptions::default();
  options
    .set_allow_latest(allow_latest)
    .set_allow_insecure(allowed_insecure.to_vec());
  WickWasmModule::from_slice(&location.bytes(&options).await?)
}
