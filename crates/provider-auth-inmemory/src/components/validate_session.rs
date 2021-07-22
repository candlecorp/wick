use vino_interfaces_authentication::validate_session::*;
use vino_provider::error::ProviderComponentError;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> Result<(), Box<ProviderComponentError>> {
  let state = context.lock().unwrap();
  let user_id = state
    .sessions
    .get(&input.session)
    .ok_or_else(|| ProviderComponentError::new(format!("Session '{}' not found", input.session)))?
    .clone();
  drop(state);
  output.user_id.done(user_id);
  Ok(())
}
