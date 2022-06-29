use wasmflow_interface_keyvalue::exists::*;

use crate::components::generated::exists::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    trace!(?input, "exists");
    let mut cmd = redis::Cmd::exists(&input.key);
    let value: u32 = context.run_cmd(&mut cmd).await?;
    if value == 0 {
      output.exists.done(false)?;
    } else {
      output.exists.done(true)?;
    };
    Ok(())
  }
}
