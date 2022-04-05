use vino_random::Random;

pub use crate::components::generated::rand::uuid::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, _context: crate::Context) -> JobResult {
  let rng = Random::from_seed(vino_random::Seed::unsafe_new(input.seed));
  output
    .output
    .done(Payload::success(&rng.uuid().to_hyphenated().to_string()))?;
  Ok(())
}
