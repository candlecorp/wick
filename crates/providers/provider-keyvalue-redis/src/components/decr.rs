pub use wafl_interface_keyvalue::decr::*;

use crate::components::generated::decr::*;
pub use crate::components::generated::decr::*;

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
    trace!(?input, "decr");

    let mut cmd = redis::Cmd::decr(&input.key, input.amount);
    let value: String = context.run_cmd(&mut cmd).await?;
    let num: i64 = value
      .parse()
      .map_err(|_| format!("Could not parse string into integer. Value was '{}' ", value))?;
    output.output.done(num)?;

    Ok(state)
  }
}
