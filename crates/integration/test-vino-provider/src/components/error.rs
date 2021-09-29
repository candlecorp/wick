use vino_provider::native::prelude::*;

use crate::generated::error::*;

pub(crate) async fn job(
  input: Inputs,
  output: OutputPorts,
  _context: crate::Context,
) -> Result<(), Box<NativeComponentError>> {
  Err(Box::new(NativeComponentError::new("This always errors")))
}
