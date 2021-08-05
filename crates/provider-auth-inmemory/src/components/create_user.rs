use bcrypt::{
  hash,
  DEFAULT_COST,
};
use vino_interface_authentication::create_user::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let hashed = hash(input.password, DEFAULT_COST)
    .map_err(|e| NativeComponentError::new(format!("Hashing failed: {}", e.to_string())))?;
  let mut state = context.lock().unwrap();
  if state.user_ids.contains_key(&input.username) {
    return Err(NativeComponentError::new(format!(
      "Username '{}' exists",
      input.username
    )));
  }
  if state.auth.contains_key(&input.user_id) {
    return Err(NativeComponentError::new(format!(
      "User ID '{}' exists",
      input.user_id
    )));
  }
  state
    .user_ids
    .insert(input.username.clone(), input.user_id.clone());
  state.auth.insert(input.user_id.clone(), hashed);
  drop(state);
  output.user_id.done(&input.user_id)?;
  Ok(())
}
