pub use crate::components::generated::gate::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, _context: crate::Context) -> JobResult {
  if input.condition {
    output.output.done(input.value)?;
  } else {
    output.output.done(Payload::exception(input.exception))?;
  }
  Ok(())
}
