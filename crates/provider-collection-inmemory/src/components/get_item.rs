pub(crate) use vino_interface_collection::get_item::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let state = context.lock().unwrap();
  let content_key = format!("{}:{}", input.collection_id, input.document_id);
  match state.documents.get(&content_key) {
    Some(content) => output.document.done(content)?,
    None => output
      .document
      .done_exception(format!("No content with id {} found", content_key))?,
  };
  Ok(())
}
