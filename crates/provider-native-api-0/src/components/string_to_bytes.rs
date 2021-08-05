use crate::generated::string_to_bytes::*;

pub(crate) async fn job(input: Inputs, output: Outputs, _context: crate::Context) -> JobResult {
  output.output.done(&input.input.into_bytes())?;
  Ok(())
}