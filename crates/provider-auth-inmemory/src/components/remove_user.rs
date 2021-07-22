use vino_interfaces_authentication::remove_user::*;
use vino_provider::error::ProviderComponentError;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> Result<(), Box<ProviderComponentError>> {
  let mut state = context.lock().unwrap();
  if let Some(uid) = state.user_ids.remove(&input.username) {
    state.auth.remove(&uid);
    output.user_id.done(uid);
    Ok(())
  } else {
    Err(Box::new(ProviderComponentError::new(format!(
      "User {} not found",
      input.username
    ))))
  }
}
