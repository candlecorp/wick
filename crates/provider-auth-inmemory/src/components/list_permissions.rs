use vino_interfaces_authentication::list_permissions::*;
use vino_provider::error::ProviderComponentError;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> Result<(), Box<ProviderComponentError>> {
  let state = context.lock().unwrap();
  if let Some(perms) = state.permissions.get(&input.user_id) {
    output.permissions.done(perms.clone());
  } else {
    output.permissions.done(vec![]);
  }
  Ok(())
}
