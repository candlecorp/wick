use crate::generated::error::*;

pub(crate) fn job(input: Inputs, output: Outputs) -> JobResult {
  console_log("About to panic!");
  panic!("This component always panics");
  Ok(())
}
