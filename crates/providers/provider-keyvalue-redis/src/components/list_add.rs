use wafl_interface_keyvalue::list_add::*;

use crate::components::generated::list_add::*;

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
    trace!(?input, "list-add");
    let mut cmd = redis::Cmd::rpush(&input.key, &input.values);
    let value: u32 = context.run_cmd(&mut cmd).await?;
    output.length.done(value)?;

    Ok(state)
  }
}
