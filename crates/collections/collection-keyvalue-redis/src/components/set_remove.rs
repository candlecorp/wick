use wick_interface_types_keyvalue::set_remove::*;

use crate::components::generated::set_remove::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    trace!(?input, "set-remove");
    let mut cmd = redis::Cmd::srem(&input.key, &input.values);
    let num: u32 = context.run_cmd(&mut cmd).await?;
    output.num.done(num)?;
    Ok(())
  }
}
