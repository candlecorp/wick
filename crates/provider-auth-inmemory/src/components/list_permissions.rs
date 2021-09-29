use vino_interface_authentication::list_permissions::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let state = context.lock().unwrap();
  if let Some(perms) = state.permissions.get(&input.user_id) {
    output.permissions.done(Payload::success(perms))?;
  } else {
    output
      .permissions
      .done(Payload::success::<Vec<String>>(Vec::new().as_ref()))?;
  }
  Ok(())
}
