use crate::generated::add::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> JobResult {
  let result = input.left + input.right;
  output.output.done(&result)?;
  Ok(())
}
