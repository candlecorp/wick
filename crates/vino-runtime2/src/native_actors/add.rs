use crate::native_actor;

native_actor! {
  add,
  fn job(input: Inputs, output: Outputs) -> Result<Signal> {
  trace!("in job");

        output.output.send(input.left + input.right)?;
  trace!("post send");
        Ok(Signal::Done)
    }
}
