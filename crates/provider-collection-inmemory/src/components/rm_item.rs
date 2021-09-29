pub(crate) use vino_interface_collection::rm_item::*;

pub(crate) async fn job(
  input: Inputs,
  _output: OutputPorts,
  context: crate::Context,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let mut state = context.lock().unwrap();
  let content_key = format!("{}:{}", input.collection_id, input.document_id);
  state.documents.remove(&content_key);
  Ok(())
}
