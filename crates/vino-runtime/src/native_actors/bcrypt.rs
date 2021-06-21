use bcrypt::hash;
use vino_provider::provider_component;

use crate::Result;

provider_component! {
  bcrypt,
  fn job(input: Inputs, output: Outputs, _context: Context<super::State>) -> Result<()> {
        let hashed = hash(input.input, input.cost)?;
        output.output.send(hashed);
        Ok(())
    }
}
