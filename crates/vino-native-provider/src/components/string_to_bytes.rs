use vino_provider::Context;
use vino_rpc::port::Sender;

pub(crate) use super::generated::string_to_bytes::{
  Inputs,
  Outputs,
};

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
  output.output.send(input.input.into_bytes());
  Ok(())
}
