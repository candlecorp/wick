use crate::native_actor;

native_actor! {
  add,
  fn job(input: Inputs, output: Outputs) -> Result<Signal> {
        output.output.send(input.left + input.right)?;
        Ok(Signal::Done)
    }
}
