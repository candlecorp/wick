use crate::generated::negate::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, _context: crate::Context) -> JobResult {
  output.output.done(Payload::success(&!input.input))?;
  Ok(())
}
