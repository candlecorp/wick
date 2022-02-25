pub use crate::components::generated::core::panic::*;

pub(crate) async fn job(
  _input: Inputs,
  _output: OutputPorts,
  _context: crate::Context,
) -> Result<(), Box<NativeComponentError>> {
  panic!("This component will always panic");
}
