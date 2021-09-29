use vino_interface_authentication::remove_user::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut state = context.lock().unwrap();
  if let Some(uid) = state.user_ids.remove(&input.username) {
    state.auth.remove(&uid);
    output.user_id.done(Payload::success(&uid))?;
    Ok(())
  } else {
    Err(format!("User {} not found", input.username).into())
  }
}
