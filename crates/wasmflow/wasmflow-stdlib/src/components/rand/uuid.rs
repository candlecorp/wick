use seeded_random::Random;

pub use crate::components::generated::rand::uuid::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    _context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rng = Random::from_seed(seeded_random::Seed::unsafe_new(input.seed));
    output.output.done(rng.uuid().as_hyphenated().to_string())?;
    Ok(())
  }
}
