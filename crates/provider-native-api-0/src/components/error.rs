use crate::generated::error::*;

pub(crate) async fn job(_input: Inputs, _output: Outputs, _context: crate::Context) -> JobResult {
  panic!("This component will always panic");
}
