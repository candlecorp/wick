pub use wasmflow_interface_keyvalue::incr::*;

use crate::components::generated::incr::*;

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
    trace!(?input, "incr");

    let mut cmd = redis::Cmd::incr(&input.key, input.amount);
    let value: String = context.run_cmd(&mut cmd).await?;
    let num: i64 = value
      .parse()
      .map_err(|_| format!("Could not parse string into integer. Value was '{}' ", value))?;
    output.output.done(num)?;

    Ok(state)
  }
}
