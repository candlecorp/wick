use vino_provider::Context;
use vino_rpc::port::Sender;

pub(crate) use super::generated::log::{
  Inputs,
  Outputs,
};

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
  println!("Logger: {}", input.input);
  output.output.done(input.input);
  Ok(())
}
