use vino_provider::provider_component;

use crate::State;

provider_component! {
  test,
  fn job(input: Inputs, output: Outputs, _context: Context<State>) -> Result<()> {
        output.output.done(input.input);
        Ok(())
    }
}
