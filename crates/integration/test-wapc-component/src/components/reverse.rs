pub use crate::components::generated::reverse::*;

pub(crate) fn job(input: Inputs, output: OutputPorts) -> JobResult {
  let reversed = input.input.chars().rev().collect();
  output.output.done(&reversed)?;
  Ok(())
}
