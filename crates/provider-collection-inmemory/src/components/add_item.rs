pub(crate) use vino_interfaces_collection::add_item::*;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let mut state = context.lock().unwrap();
  let content_key = format!("{}:{}", input.collection_id, input.document_id);
  state.documents.insert(content_key, input.document);
  let list = state
    .collections
    .entry(input.collection_id)
    .or_insert_with(Vec::new);
  list.push(input.document_id.to_string());
  output.document_id.done(input.document_id);
  Ok(())
}
