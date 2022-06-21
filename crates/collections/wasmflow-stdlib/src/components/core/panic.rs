pub use crate::components::generated::core::panic::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;
  type State = State;
  async fn job(
    _input: Self::Inputs,
    _output: Self::Outputs,
    _context: Self::Context,
    _state: Option<Self::State>,
    _config: Option<Self::Config>,
  ) -> Result<Option<Self::State>, Box<dyn std::error::Error + Send + Sync>> {
    panic!("This component will always panic");
  }
}
