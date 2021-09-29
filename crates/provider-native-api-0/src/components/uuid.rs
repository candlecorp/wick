use crate::generated::uuid::*;

pub(crate) async fn job(
  _input: Inputs,
  output: OutputPorts,
  _context: crate::Context,
) -> JobResult {
  output
    .output
    .done(Payload::success(&uuid::Uuid::new_v4().to_string()))?;
  Ok(())
}
