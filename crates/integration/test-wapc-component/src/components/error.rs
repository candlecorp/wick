use crate::generated::error::*;

pub(crate) fn job(_input: Inputs, _output: Outputs) -> JobResult {
  console_log("About to panic");
  panic!("This WaPC component will always panic")
}
