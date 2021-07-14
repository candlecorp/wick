pub(crate) use vino_interfaces_collection::get_item::*;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let state = context.lock().unwrap();
  let content_key = format!("{}:{}", input.collection_id, input.document_id);
  match state.documents.get(&content_key) {
    Some(content) => output.document.done(content.to_string()),
    None => output
      .document
      .done_exception(format!("No content with id {} found", content_key)),
  };
  Ok(())
}
