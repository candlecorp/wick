pub use crate::components::generated::byte_count::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::ephemeral::BatchedComponent for Component {
  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let num_bytes = input.input.len().try_into()?;
    output.output.done(num_bytes)?;
    Ok(())
  }
}
