pub use crate::components::generated::__batch__::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::v1::ephemeral::BatchedComponent for Component {
  type State = State;
  async fn job(
    _input: Self::Inputs,
    _output: Self::Outputs,
    _state: Option<Self::State>,
    _config: Option<Self::Config>,
  ) -> Result<Option<Self::State>, Box<dyn std::error::Error + Send + Sync>> {
    unimplemented!();
  }
}
