use vino_random::Random;

pub use crate::components::generated::uuid::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, _context: crate::Context) -> JobResult {
  let rng = Random::from_seed(input.seed);
  output.output.done(Payload::success(&rng.get_uuid()))?;
  Ok(())
}
