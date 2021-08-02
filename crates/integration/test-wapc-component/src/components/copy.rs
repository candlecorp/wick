use crate::generated::copy::*;

pub(crate) fn job(input: Inputs, output: Outputs) -> JobResult {
  for _ in 0..(input.times) {
    output.output.send(&input.input)?;
  }
  output.output.close()?;
  Ok(())
}
