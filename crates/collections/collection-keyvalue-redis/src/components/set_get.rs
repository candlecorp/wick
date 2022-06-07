use wasmflow_interface_keyvalue::set_get::*;

use crate::components::generated::set_get::*;

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
    trace!(?input, "set-get");
    let mut cmd = redis::Cmd::smembers(&input.key);
    let values: Vec<String> = context.run_cmd(&mut cmd).await?;
    output.values.done(values)?;
    Ok(state)
  }
}
