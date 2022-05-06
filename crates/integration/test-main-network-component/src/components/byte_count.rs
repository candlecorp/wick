pub use crate::components::generated::byte_count::*;

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
    let num_bytes = input.input.len().try_into()?;
    output.output.done(num_bytes)?;
    Ok(state)
  }
}
