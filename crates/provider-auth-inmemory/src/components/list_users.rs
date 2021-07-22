use std::collections::HashMap;
use std::convert::TryInto;

use vino_interfaces_authentication::list_users::*;
use vino_provider::error::ProviderComponentError;
use vino_provider::Context;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  context: Context<crate::State>,
) -> Result<(), Box<ProviderComponentError>> {
  let state = context.lock().unwrap();
  let users: HashMap<String, String> = state
    .user_ids
    .iter()
    .skip(input.offset.try_into().map_err(|_| {
      ProviderComponentError::new(format!("Could not convert {} to usize", input.offset))
    })?)
    .take(input.limit.try_into().map_err(|_| {
      ProviderComponentError::new(format!("Could not convert {} to usize", input.limit))
    })?)
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect();
  output.users.done(users);
  Ok(())
}
