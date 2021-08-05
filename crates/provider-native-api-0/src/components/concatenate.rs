use crate::generated::concatenate::*;

pub(crate) async fn job(input: Inputs, output: Outputs, _context: crate::Context) -> JobResult {
  output
    .output
    .done(&format!("{}{}", input.left, input.right))?;
  Ok(())
}
