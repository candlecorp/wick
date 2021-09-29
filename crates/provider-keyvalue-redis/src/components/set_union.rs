use vino_interface_keyvalue::generated::set_union::*;
use vino_provider::native::prelude::*;

pub(crate) async fn job(
  input: Inputs,
  output: OutputPorts,
  _context: crate::Context,
) -> Result<(), Box<NativeComponentError>> {
  Ok(())
}
