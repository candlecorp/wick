use crate::generated::error::*;

pub(crate) async fn job(
  _input: Inputs,
  _output: Outputs,
  _context: Context<crate::State>,
) -> JobResult {
  panic!("This component will always panic");
}
