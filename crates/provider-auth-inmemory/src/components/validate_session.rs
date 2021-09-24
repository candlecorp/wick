use vino_interface_authentication::validate_session::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let state = context.lock().unwrap();
  let user_id = state
    .sessions
    .get(&input.session)
    .ok_or_else(|| NativeComponentError::new(format!("Session '{}' not found", input.session)))?
    .clone();
  drop(state);
  output.user_id.done(Payload::success(&user_id))?;
  Ok(())
}
