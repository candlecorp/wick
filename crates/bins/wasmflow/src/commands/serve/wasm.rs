use std::sync::Arc;

use anyhow::Result;
use wasmflow_collection_wasm::collection::{Collection, WasiParams};
use wasmflow_collection_wasm::helpers::WapcModule;

pub(crate) async fn handle_command(opts: super::ServeCommand, bytes: Vec<u8>) -> Result<()> {
  let component = WapcModule::from_slice(&bytes)?;

  let wasi: WasiParams = (&opts.wasi).into();
  let collection = Arc::new(
    match Collection::try_load(&component, 1, None, Some(wasi.clone()), None) {
      Ok(collection) => collection,
      Err(e) => {
        error!("Error starting WebAssembly collection: {}", e);
        panic!();
      }
    },
  );

  wasmflow_collection_cli::init_cli(collection.clone(), Some(opts.cli.into())).await?;

  Ok(())
}
