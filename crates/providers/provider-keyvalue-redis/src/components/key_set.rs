use wafl_interface_keyvalue::key_set::*;

use crate::components::generated::key_set::*;

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
    trace!(?input, "key-set");
    if input.expires != 0 {
      let mut cmd = redis::Cmd::set_ex(&input.key, &input.value, input.expires as _);
      let _value: String = context.run_cmd(&mut cmd).await?;
    } else {
      let mut cmd = redis::Cmd::set(&input.key, &input.value);
      let _value: String = context.run_cmd(&mut cmd).await?;
    }
    output.result.done(true)?;
    Ok(state)
  }
}
