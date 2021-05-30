use crate::native_actor;

native_actor! {
  log,
  fn job(input: Inputs, output: Outputs) -> Result<Signal> {
        println!("Logger: {}", input.input);
        output.output.send(input.input)?;
        Ok(Signal::Done)
    }
}
