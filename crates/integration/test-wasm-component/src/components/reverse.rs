pub use crate::components::generated::reverse::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::v1::ephemeral::BatchedComponent for Component {
  type State = State;
  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    state: Option<Self::State>,
    _config: Option<Self::Config>,
  ) -> Result<Option<Self::State>, Box<dyn std::error::Error + Send + Sync>> {
    let reversed = input.input.chars().rev().collect();
    output.output.done(reversed)?;
    Ok(state)
  }
}
