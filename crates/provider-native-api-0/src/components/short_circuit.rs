use crate::generated::short_circuit::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> JobResult {
  output.output.done_exception(input.input)?;
  Ok(())
}
