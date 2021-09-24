use crate::generated::add::*;

pub(crate) async fn job(input: Inputs, output: Outputs, _context: crate::Context) -> JobResult {
  let result = input.left + input.right;
  output.output.done(Payload::success(&result))?;
  Ok(())
}
