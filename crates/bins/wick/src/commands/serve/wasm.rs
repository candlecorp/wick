use std::sync::Arc;

use anyhow::Result;
use wick_component_wasm::collection::Collection;
use wick_component_wasm::helpers::WickWasmModule;
use wick_config_component::Permissions;

pub(crate) async fn handle_command(opts: super::ServeCommand, bytes: Vec<u8>) -> Result<()> {
  let component = WickWasmModule::from_slice(&bytes)?;

  let perms: Permissions = (opts.wasi).into();
  let collection = Arc::new(
    match Collection::try_load(&component, 1, None, Some(perms.clone()), None) {
      Ok(collection) => collection,
      Err(e) => {
        error!("Error starting WebAssembly collection: {}", e);
        panic!();
      }
    },
  );

  wick_component_cli::init_cli(collection.clone(), Some(opts.cli.into())).await?;

  Ok(())
}
