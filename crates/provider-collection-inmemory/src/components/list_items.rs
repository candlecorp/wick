pub(crate) use vino_interfaces_collection::list_items::*;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let mut state = context.lock().unwrap();
  let list = state.collections.entry(input.collection_id).or_default();
  output.document_ids.done(list.clone());
  Ok(())
}
