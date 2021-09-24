use bcrypt::verify;
use vino_interface_authentication::authenticate::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let mut state = context.lock().unwrap();
  let uid = state
    .user_ids
    .get(&input.username)
    .cloned()
    .ok_or_else(|| format!("Username '{}' not found", input.username))?;
  let expected_hash = state
    .auth
    .get(&uid)
    .ok_or_else(|| format!("User ID '{}' not found", uid))?;
  let valid = verify(input.password, expected_hash)
    .map_err(|e| format!("Hashing failed: {}", e.to_string()))?;
  if valid {
    state.sessions.insert(input.session.clone(), uid.clone());
    output.user_id.done(Payload::success(&uid))?;
    output.session.done(Payload::success(&input.session))?;
    Ok(())
  } else {
    Err("Invalid password".into())
  }
}
