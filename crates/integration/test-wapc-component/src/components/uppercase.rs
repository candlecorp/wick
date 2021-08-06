use crate::generated::uppercase::*;

pub(crate) fn job(input: Inputs, output: Outputs) -> JobResult {
  output.output.done(&input.input.to_uppercase())?;
  Ok(())
}
