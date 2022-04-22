pub use crate::components::generated::byte_count::*;

pub(crate) fn job(input: Inputs, output: OutputPorts) -> JobResult {
  output
    .output
    .done(&input.input.len().try_into().map_err(ComponentError::new)?)?;
  Ok(())
}
