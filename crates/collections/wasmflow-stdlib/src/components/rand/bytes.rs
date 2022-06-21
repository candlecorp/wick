use std::convert::TryInto;

use seeded_random::Random;

pub use crate::components::generated::rand::bytes::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;
  type State = State;
  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    _context: Self::Context,
    state: Option<Self::State>,
    _config: Option<Self::Config>,
  ) -> Result<Option<Self::State>, Box<dyn std::error::Error + Send + Sync>> {
    let num: usize = input.length.try_into().unwrap();
    let rng = Random::from_seed(seeded_random::Seed::unsafe_new(input.seed));
    let bytes = rng.bytes(num);
    output.output.done(bytes)?;
    Ok(state)
  }
}
