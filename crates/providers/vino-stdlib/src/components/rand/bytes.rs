use std::convert::TryInto;

use vino_random::Random;

pub use crate::components::generated::rand::bytes::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, _context: crate::Context) -> JobResult {
  let num: usize = input.length.try_into().map_err::<NativeComponentError, _>(|_| {
    format!("Invalid number ({}) passed to random-bytes", input.length).into()
  })?;
  let rng = Random::from_seed(vino_random::Seed::unsafe_new(input.seed));
  let bytes = rng.bytes(num);
  output.output.done(Payload::success(&bytes))?;
  Ok(())
}
