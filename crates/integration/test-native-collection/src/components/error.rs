pub(crate) use crate::components::generated::error::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    _input: Self::Inputs,
    _output: Self::Outputs,
    _context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err("This always errors".into())
  }
}
