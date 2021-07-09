use vino_provider::Context;

use crate::generated::add::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  output.output.send(input.left + input.right);
  Ok(())
}
