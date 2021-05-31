use crate::native_actor;

native_actor! {
  string_to_bytes,
  fn job(input: Inputs, output: Outputs) -> Result<Signal> {
    trace!("hey");
        output.output.send(input.input.into_bytes())?;
        Ok(Signal::Done)
    }
}
