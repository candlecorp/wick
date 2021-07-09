use vino_provider::Context;
use vino_rpc::port::Sender;

use crate::generated::string_to_bytes::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  output.output.send(input.input.into_bytes());
  Ok(())
}
