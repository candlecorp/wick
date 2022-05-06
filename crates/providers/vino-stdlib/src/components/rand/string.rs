use std::convert::TryInto;

use vino_random::Random;

pub use crate::components::generated::rand::string::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::sdk::stateful::BatchedComponent for Component {
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
    let rng = Random::from_seed(vino_random::Seed::unsafe_new(input.seed));
    let string = rng.string(num);
    output.output.done(string)?;
    Ok(state)
  }
}
