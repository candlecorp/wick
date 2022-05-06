use std::convert::TryInto;

use vino_interface_keyvalue::list_range::*;

use crate::components::generated::list_range::*;

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
    trace!(?input, "list-range");
    let mut cmd = redis::Cmd::lrange(
      &input.key,
      input.start.try_into().unwrap(),
      input.end.try_into().unwrap(),
    );
    let docs: Vec<String> = context.run_cmd(&mut cmd).await?;
    output.values.done(docs)?;

    Ok(state)
  }
}
