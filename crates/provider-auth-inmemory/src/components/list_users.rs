use std::collections::HashMap;
use std::convert::TryInto;

use vino_interface_authentication::list_users::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let state = context.lock().unwrap();
  let users: HashMap<String, String> = state
    .user_ids
    .iter()
    .skip(input.offset.try_into().map_err(|_| {
      NativeComponentError::new(format!("Could not convert {} to usize", input.offset))
    })?)
    .take(input.limit.try_into().map_err(|_| {
      NativeComponentError::new(format!("Could not convert {} to usize", input.limit))
    })?)
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect();
  output.users.done(&users)?;
  Ok(())
}
