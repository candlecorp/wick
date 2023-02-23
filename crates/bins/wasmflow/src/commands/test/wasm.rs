use std::sync::Arc;

use anyhow::Result;
use wasmflow_collection_wasm::collection::Collection;
use wasmflow_collection_wasm::helpers::WickWasmModule;
use wasmflow_test::TestSuite;

#[allow(clippy::future_not_send, clippy::too_many_lines)]
pub(crate) async fn handle_command(opts: super::TestCommand, bytes: Vec<u8>) -> Result<()> {
  let component = WickWasmModule::from_slice(&bytes)?;

  let collection = Collection::try_load(&component, 1, None, Some((opts.wasi).into()), None)?;

  let mut suite = TestSuite::try_from_file(opts.data_path.clone())?;

  let runner = suite.run(Arc::new(collection)).await?;
  runner.print();

  Ok(())
}
