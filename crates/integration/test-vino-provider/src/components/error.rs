pub(crate) use crate::components::generated::error::*;

pub(crate) async fn job(
  _input: Inputs,
  _output: OutputPorts,
  _context: crate::Context,
) -> Result<(), Box<NativeComponentError>> {
  Err(Box::new(NativeComponentError::new("This always errors")))
}
