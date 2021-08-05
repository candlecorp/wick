use vino_interface_authentication::has_permission::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let state = context.lock().unwrap();
  if let Some(perms) = state.permissions.get(&input.user_id) {
    if perms.contains(&input.permission) {
      output.user_id.done(&input.user_id)?;
      return Ok(());
    }
  }
  output.user_id.done_exception(format!(
    "User ID '{}' does not have permission '{}'",
    input.user_id, input.permission
  ))?;
  Ok(())
}
