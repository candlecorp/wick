use vino_provider::error::ProviderComponentError;
use vino_provider::Context;

use crate::generated::concatenate::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> Result<(), Box<ProviderComponentError>> {
  output.output.done(format!("{}{}", input.left, input.right));
  Ok(())
}
