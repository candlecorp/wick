pub use crate::components::generated::error::*;

pub(crate) fn job(input: Inputs, output: OutputPorts) -> JobResult {
  console_log("About to panic!");
  panic!("This component always panics");
  Ok(())
}
