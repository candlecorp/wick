pub use crate::components::generated::reverse::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::ephemeral::BatchedComponent for Component {
  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let reversed = input.input.chars().rev().collect();
    output.output.done(reversed)?;
    Ok(())
  }
}
