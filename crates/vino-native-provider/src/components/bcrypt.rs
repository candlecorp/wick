use bcrypt::hash;
use vino_provider::provider_component;

use crate::Result;

provider_component! {
  bcrypt,
  fn job(input: Inputs, output: Outputs, _context: Context<crate::State>) -> Result<()> {
        let hashed = hash(input.input, input.cost).map_err(|e|crate::error::NativeError::Other(format!("Error generating hash : {}", e.to_string())))?;
        output.output.send(hashed);
        Ok(())
    }
}
