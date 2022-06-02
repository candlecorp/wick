use std::sync::Arc;

use anyhow::Result;
use wasmflow_collection_wasm::helpers::WapcModule;
use wasmflow_collection_wasm::provider::{Provider, WasiParams};

pub(crate) async fn handle_command(opts: super::ServeCommand, bytes: Vec<u8>) -> Result<()> {
  let component = WapcModule::from_slice(&bytes)?;

  let wasi: WasiParams = (&opts.wasi).into();
  let provider = Arc::new(
    match Provider::try_load(&component, 1, None, Some(wasi.clone()), None) {
      Ok(provider) => provider,
      Err(e) => {
        error!("Error starting WebAssembly provider: {}", e);
        panic!();
      }
    },
  );

  wasmflow_collection_cli::init_cli(provider.clone(), Some(opts.cli.into())).await?;

  Ok(())
}
