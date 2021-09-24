use vino_interface_authentication::get_id::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let state = context.lock().unwrap();
  let user_id = state
    .user_ids
    .get(&input.username)
    .ok_or_else(|| NativeComponentError::new(format!("User '{}' not found", input.username)))?;
  output.user_id.done(Payload::success(user_id))?;
  Ok(())
}
