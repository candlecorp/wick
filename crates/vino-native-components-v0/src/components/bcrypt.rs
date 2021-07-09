use bcrypt::hash;
use vino_provider::Context;

use crate::generated::bcrypt::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let hashed = hash(input.input, input.cost).map_err(|e| {
    crate::error::NativeError::Other(format!("Error generating hash : {}", e.to_string()))
  })?;
  output.output.send(hashed);
  Ok(())
}
