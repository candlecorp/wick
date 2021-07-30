use vino_interfaces_authentication::remove_user::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> JobResult {
  let mut state = context.lock().unwrap();
  if let Some(uid) = state.user_ids.remove(&input.username) {
    state.auth.remove(&uid);
    output.user_id.done(&uid)?;
    Ok(())
  } else {
    Err(format!("User {} not found", input.username).into())
  }
}
