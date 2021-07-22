use bcrypt::verify;
use vino_interfaces_authentication::authenticate::*;
use vino_provider::error::ProviderComponentError;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> Result<(), Box<ProviderComponentError>> {
  let mut state = context.lock().unwrap();
  let uid = state
    .user_ids
    .get(&input.username)
    .cloned()
    .ok_or_else(|| {
      ProviderComponentError::new(format!("Username '{}' not found", input.username))
    })?;
  let expected_hash = state
    .auth
    .get(&uid)
    .ok_or_else(|| ProviderComponentError::new(format!("User ID '{}' not found", uid)))?;
  let valid = verify(input.password, expected_hash)
    .map_err(|e| ProviderComponentError::new(format!("Hashing failed: {}", e.to_string())))?;
  if valid {
    state.sessions.insert(input.session.clone(), uid.clone());
    output.user_id.done(uid.clone());
    output.session.done(input.session);
    Ok(())
  } else {
    Err(Box::new(ProviderComponentError::new(
      "Invalid password".to_owned(),
    )))
  }
}
