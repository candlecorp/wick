pub use crate::components::generated::error::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::ephemeral::BatchedComponent for Component {
  async fn job(
    _input: Self::Inputs,
    _output: Self::Outputs,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    console_log!("About to panic!");
    panic!("This component always panics");
  }
}
