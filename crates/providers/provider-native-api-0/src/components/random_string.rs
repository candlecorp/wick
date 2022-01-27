use std::convert::TryInto;

use vino_random::Random;

pub use crate::components::generated::random_string::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, _context: crate::Context) -> JobResult {
  let num: usize = input.length.try_into().map_err::<NativeComponentError, _>(|_| {
    format!("Invalid length ({}) passed to random-string", input.length).into()
  })?;
  let rng = Random::from_seed(input.seed);
  let string = rng.get_string(num);
  output.output.done(Payload::success(&string))?;
  Ok(())
}
