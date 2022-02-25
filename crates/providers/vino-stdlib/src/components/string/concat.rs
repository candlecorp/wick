pub use crate::components::generated::string::concat::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, _context: crate::Context) -> JobResult {
  output
    .output
    .done(Payload::success(&format!("{}{}", input.left, input.right)))?;
  Ok(())
}
