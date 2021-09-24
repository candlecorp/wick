use vino_interface_authentication::update_permissions::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let mut state = context.lock().unwrap();
  state
    .permissions
    .insert(input.user_id, input.permissions.clone());
  output
    .permissions
    .done(Payload::success(&input.permissions))?;
  Ok(())
}
