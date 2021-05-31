use crate::native_actor;
use bcrypt::hash;

native_actor! {
  bcrypt,
  fn job(input: Inputs, output: Outputs) -> Result<Signal> {
        let hashed = hash(input.input, input.cost)?;
        output.output.send(hashed)?;
        Ok(Signal::Done)
    }
}
