use vino_provider::Context;

use crate::generated::error::*;

pub(crate) async fn job(
  _input: Inputs,
  _output: Outputs,
  _context: Context<crate::State>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  panic!("This component will always panic");
}
