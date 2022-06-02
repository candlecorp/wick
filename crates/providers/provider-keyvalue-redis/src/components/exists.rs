use wafl_interface_keyvalue::exists::*;

use crate::components::generated::exists::*;

pub(crate) type State = ();

#[async_trait::async_trait]
impl wasmflow_sdk::sdk::stateful::BatchedComponent for Component {
  type Context = crate::Context;
  type State = State;
  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    context: Self::Context,
    state: Option<Self::State>,
    _config: Option<Self::Config>,
  ) -> Result<Option<Self::State>, Box<dyn std::error::Error + Send + Sync>> {
    trace!(?input, "exists");
    let mut cmd = redis::Cmd::exists(&input.key);
    let value: u32 = context.run_cmd(&mut cmd).await?;
    if value == 0 {
      output.exists.done(false)?;
    } else {
      output.exists.done(true)?;
    };
    Ok(state)
  }
}
