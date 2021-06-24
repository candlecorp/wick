use vino_provider::provider_component;

use crate::Result;

provider_component! {
  log,
  fn job(input: Inputs, output: Outputs, _context: Context<crate::State>) -> Result<()> {
        println!("Logger: {}", input.input);
        output.output.send(input.input);
        Ok(())
    }
}
