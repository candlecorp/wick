use vino_provider::Context;

use crate::generated::short_circuit::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  output.output.done_exception(input.input);
  Ok(())
}
