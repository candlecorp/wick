use vino_interface_authentication::list_permissions::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let state = context.lock().unwrap();
  if let Some(perms) = state.permissions.get(&input.user_id) {
    output.permissions.done(perms)?;
  } else {
    output.permissions.done(vec![].as_ref())?;
  }
  Ok(())
}
