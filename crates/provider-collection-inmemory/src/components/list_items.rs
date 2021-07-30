pub(crate) use vino_interfaces_collection::list_items::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> JobResult {
  let mut state = context.lock().unwrap();
  let list = state.collections.entry(input.collection_id).or_default();
  output.document_ids.done(&list)?;
  Ok(())
}
