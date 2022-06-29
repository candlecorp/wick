use wasmflow_interface_keyvalue::key_set::*;

use crate::components::generated::key_set::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    trace!(?input, "key-set");
    if input.expires != 0 {
      let mut cmd = redis::Cmd::set_ex(&input.key, &input.value, input.expires as _);
      let _value: String = context.run_cmd(&mut cmd).await?;
    } else {
      let mut cmd = redis::Cmd::set(&input.key, &input.value);
      let _value: String = context.run_cmd(&mut cmd).await?;
    }
    output.result.done(true)?;
    Ok(())
  }
}
