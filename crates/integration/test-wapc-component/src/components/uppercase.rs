pub use crate::components::generated::uppercase::*;

pub(crate) fn job(input: Inputs, output: OutputPorts) -> JobResult {
  output.output.done(&input.input.to_uppercase())?;
  Ok(())
}
