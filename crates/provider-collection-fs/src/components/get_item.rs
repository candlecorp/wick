use std::fs;

pub(crate) use vino_interface_collection::get_item::*;

pub(crate) async fn job(
  input: Inputs,
  output: OutputPorts,
  context: crate::Context,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let state = context.lock().unwrap();
  let mut path = state.directory.clone();
  drop(state);
  path.push(input.collection_id);
  path.push(input.document_id);
  if !path.exists() {
    output
      .document
      .done_exception(format!("No document in path {}", path.to_string_lossy()))?;
    return Ok(());
  }
  let contents = fs::read_to_string(path)?;
  output.document.done(Payload::success(&contents))?;
  Ok(())
}
