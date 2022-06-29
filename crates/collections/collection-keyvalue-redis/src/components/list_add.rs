use wasmflow_interface_keyvalue::list_add::*;

use crate::components::generated::list_add::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    trace!(?input, "list-add");
    let mut cmd = redis::Cmd::rpush(&input.key, &input.values);
    let value: u32 = context.run_cmd(&mut cmd).await?;
    output.length.done(value)?;

    Ok(())
  }
}
