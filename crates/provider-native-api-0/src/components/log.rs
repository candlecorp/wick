use crate::generated::log::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, _context: crate::Context) -> JobResult {
  println!("Logger: {}", input.input);
  output.output.done(Payload::success(&input.input))?;
  Ok(())
}
