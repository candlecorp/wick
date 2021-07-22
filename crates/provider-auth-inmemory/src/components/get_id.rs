use vino_interfaces_authentication::get_id::*;
use vino_provider::error::ProviderComponentError;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> Result<(), Box<ProviderComponentError>> {
  let state = context.lock().unwrap();
  let user_id = state
    .user_ids
    .get(&input.username)
    .ok_or_else(|| ProviderComponentError::new(format!("User '{}' not found", input.username)))?;
  output.user_id.done(user_id.clone());
  Ok(())
}
