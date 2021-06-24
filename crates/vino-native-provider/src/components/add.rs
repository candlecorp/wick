use vino_provider::provider_component;

use crate::Result;

provider_component! {
  add,
  fn job(input: Inputs, output: Outputs, _context: Context<crate::State>) -> Result<()> {
        output.output.send(input.left + input.right);
        Ok(())
    }
}
