pub use crate::components::generated::error::*;

pub(crate) async fn job(
  _input: Inputs,
  _output: OutputPorts,
  _context: crate::Context,
) -> JobResult {
  Err(NativeComponentError::new(
    "This component will always error",
  ))
}
