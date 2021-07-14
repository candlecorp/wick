use std::fs;

pub(crate) use vino_interfaces_collection::rm_item::*;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  _output: Outputs,
  context: Context<crate::State>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let state = context.lock().unwrap();
  let mut path = state.directory.clone();
  drop(state);
  path.push(input.collection_id);
  path.push(input.document_id);
  if path.exists() {
    fs::remove_file(path)?;
  }
  Ok(())
}
