use vino_interface_keyvalue::generated::key_increment::*;
use vino_provider::native::prelude::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: crate::Context,
) -> Result<(), Box<NativeComponentError>> {
  Ok(())
}
