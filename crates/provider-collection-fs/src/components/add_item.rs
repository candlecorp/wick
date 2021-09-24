use std::fs;

pub(crate) use vino_interface_collection::add_item::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: crate::Context,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let state = context.lock().unwrap();
  let mut path = state.directory.clone();
  drop(state);
  path.push(input.collection_id);
  if !path.exists() {
    info!("Creating directory {}", path.to_string_lossy());
    fs::create_dir_all(&path)?;
  }
  path.push(&input.document_id);
  fs::write(path, input.document)?;
  output
    .document_id
    .done(Payload::success(&input.document_id))?;
  Ok(())
}
