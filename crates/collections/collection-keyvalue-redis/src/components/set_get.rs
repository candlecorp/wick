use wasmflow_interface_keyvalue::set_get::*;

use crate::components::generated::set_get::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    trace!(?input, "set-get");
    let mut cmd = redis::Cmd::smembers(&input.key);
    let values: Vec<String> = context.run_cmd(&mut cmd).await?;
    output.values.done(values)?;
    Ok(())
  }
}
