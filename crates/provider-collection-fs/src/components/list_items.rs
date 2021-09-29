use std::fs;

pub(crate) use vino_interface_collection::list_items::*;

pub(crate) async fn job(
  input: Inputs,
  output: OutputPorts,
  context: crate::Context,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let state = context.lock().unwrap();
  let mut path = state.directory.clone();
  drop(state);
  path.push(input.collection_id);
  if !path.exists() {
    output
      .document_ids
      .done_exception(format!("No directory found at {}", path.to_string_lossy()))?;
    return Ok(());
  }

  let contents = fs::read_dir(path)?;
  let list: Vec<String> = contents
    .filter_map(Result::ok)
    .map(|dir| dir.file_name().to_string_lossy().into())
    .collect();
  output.document_ids.done(Payload::success(&list))?;
  Ok(())
}
