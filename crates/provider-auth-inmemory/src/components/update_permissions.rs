use vino_interfaces_authentication::update_permissions::*;
use vino_provider::error::ProviderComponentError;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> Result<(), Box<ProviderComponentError>> {
  let mut state = context.lock().unwrap();
  state
    .permissions
    .insert(input.user_id, input.permissions.clone());
  output.permissions.done(input.permissions);
  Ok(())
}
