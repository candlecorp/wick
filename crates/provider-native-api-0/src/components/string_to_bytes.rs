use crate::generated::string_to_bytes::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> JobResult {
  output.output.done(&input.input.into_bytes())?;
  Ok(())
}
