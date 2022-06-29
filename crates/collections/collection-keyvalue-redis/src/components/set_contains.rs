use wasmflow_interface_keyvalue::set_contains::*;

use crate::components::generated::set_contains::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    trace!(?input, "set-contains");
    let mut cmd = redis::Cmd::sismember(&input.key, &input.member);
    let exists: bool = context.run_cmd(&mut cmd).await?;
    output.exists.done(exists)?;
    Ok(())
  }
}
