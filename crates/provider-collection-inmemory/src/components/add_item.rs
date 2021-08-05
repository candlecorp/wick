pub(crate) use vino_interface_collection::add_item::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let mut state = context.lock().unwrap();
  let content_key = format!("{}:{}", input.collection_id, input.document_id);
  state.documents.insert(content_key, input.document);
  let list = state
    .collections
    .entry(input.collection_id)
    .or_insert_with(Vec::new);
  list.push(input.document_id.clone());
  output.document_id.done(&input.document_id)?;
  Ok(())
}
