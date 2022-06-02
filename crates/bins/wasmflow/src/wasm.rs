use clap::Args;
use wasmflow_collection_wasm::provider::WasiParams;

pub(crate) fn is_wasm(bytes: &[u8]) -> bool {
  let is_wasm = bytes.starts_with(&[0x00, 0x61, 0x73, 0x6d]);
  trace!(is_wasm, bytes = ?bytes[0..4], "bytes include wasm header?");
  is_wasm
}

#[derive(Debug, Clone, Args)]
pub(crate) struct WasiOptions {
  /// Directories to expose to the WASM module via WASI. Ignored if loading a manifest.
  #[clap(long = "wasi-dir")]
  wasi_dir: Vec<String>,
}

impl From<&WasiOptions> for WasiParams {
  fn from(opts: &WasiOptions) -> Self {
    WasiParams {
      preopened_dirs: opts.wasi_dir.clone(),
      ..Default::default()
    }
  }
}
