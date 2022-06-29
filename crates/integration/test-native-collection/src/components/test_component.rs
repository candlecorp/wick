pub(crate) use crate::components::generated::test_component::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    _context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    output.output.done(format!("TEST: {}", input.input))?;
    Ok(())
  }
}
