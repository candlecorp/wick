use vino_provider::error::ProviderComponentError;
use vino_provider::Context;

use crate::generated::uuid::*;

pub(crate) async fn job(
  _input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> Result<(), Box<ProviderComponentError>> {
  output.output.done(uuid::Uuid::new_v4().to_string());
  Ok(())
}
