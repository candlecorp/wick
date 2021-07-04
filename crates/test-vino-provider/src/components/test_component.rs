use vino_provider::Context;
use vino_rpc::port::Sender;

pub(crate) use super::generated::test_component::{
  Inputs,
  Outputs,
};

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
  output.output.done(format!("TEST: {}", input.input));
  Ok(())
}
