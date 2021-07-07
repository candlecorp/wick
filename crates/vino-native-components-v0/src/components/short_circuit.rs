use vino_provider::Context;
use vino_rpc::port::Sender;

pub(crate) use super::generated::short_circuit::{
  Inputs,
  Outputs,
};

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  output.output.done_exception(input.input);
  Ok(())
}
