use std::fs;

pub(crate) use vino_interfaces_collection::get_item::*;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let state = context.lock().unwrap();
  let mut path = state.directory.clone();
  drop(state);
  path.push(input.collection_id);
  path.push(input.document_id);
  if !path.exists() {
    output
      .document
      .done_exception(format!("No document in path {}", path.to_string_lossy()));
    return Ok(());
  }
  let contents = fs::read_to_string(path)?;
  output.document.done(contents);
  Ok(())
}
