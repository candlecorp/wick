use bcrypt::{
  hash,
  DEFAULT_COST,
};
use vino_interfaces_authentication::create_user::*;
use vino_provider::error::ProviderComponentError;
use vino_provider::Context;
pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> Result<(), Box<ProviderComponentError>> {
  let hashed = hash(input.password, DEFAULT_COST)
    .map_err(|e| ProviderComponentError::new(format!("Hashing failed: {}", e.to_string())))?;
  let mut state = context.lock().unwrap();
  if state.user_ids.contains_key(&input.username) {
    return Err(Box::new(ProviderComponentError::new(format!(
      "Username '{}' exists",
      input.username
    ))));
  }
  if state.auth.contains_key(&input.user_id) {
    return Err(Box::new(ProviderComponentError::new(format!(
      "User ID '{}' exists",
      input.user_id
    ))));
  }
  state
    .user_ids
    .insert(input.username.clone(), input.user_id.clone());
  state.auth.insert(input.user_id.clone(), hashed);
  drop(state);
  output.user_id.done(input.user_id);
  Ok(())
}
