use vino_provider::provider_component;

use crate::Result;

provider_component! {
  string_to_bytes,
  fn job(input: Inputs, output: Outputs, _context: Context<super::State>) -> Result<()> {
    trace!("hey");
        output.output.send(input.input.into_bytes());
        Ok(())
    }
}
