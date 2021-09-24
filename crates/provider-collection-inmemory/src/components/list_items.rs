pub(crate) use vino_interface_collection::list_items::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let mut state = context.lock().unwrap();
  let list = state.collections.entry(input.collection_id).or_default();
  output.document_ids.done(Payload::success(list))?;
  Ok(())
}
