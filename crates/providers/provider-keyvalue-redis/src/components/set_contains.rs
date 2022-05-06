use vino_interface_keyvalue::set_contains::*;

use crate::components::generated::set_contains::*;

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
    trace!(?input, "set-contains");
    let mut cmd = redis::Cmd::sismember(&input.key, &input.member);
    let exists: bool = context.run_cmd(&mut cmd).await?;
    output.exists.done(exists)?;
    Ok(state)
  }
}
