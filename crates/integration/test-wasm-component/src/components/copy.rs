pub use crate::components::generated::copy::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::sdk::ephemeral::BatchedComponent for Component {
  type State = State;
  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    state: Option<Self::State>,
    _config: Option<Self::Config>,
  ) -> Result<Option<Self::State>, Box<dyn std::error::Error + Send + Sync>> {
    for _ in 0..(input.times) {
      output.output.send(input.input.clone())?;
    }
    output.output.close()?;
    Ok(state)
  }
}
