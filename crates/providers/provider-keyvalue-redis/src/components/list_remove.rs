use std::convert::TryInto;

use vino_interface_keyvalue::list_remove::*;

use crate::components::generated::list_remove::*;
pub use crate::components::generated::list_remove::*;

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
    trace!(?input, "list-remove");
    let num: isize = input.num.try_into().unwrap_or(isize::MAX);
    let mut cmd = redis::Cmd::lrem(&input.key, num, &input.value);
    let num: u32 = context.run_cmd(&mut cmd).await?;

    output.num.done(num)?;

    Ok(state)
  }
}
