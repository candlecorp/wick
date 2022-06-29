use wasmflow_interface_keyvalue::key_get::*;

use crate::components::generated::key_get::*;
use crate::error::Exception;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    trace!(?input, "key-get");
    let mut cmd = redis::Cmd::get(&input.key);
    let value: Option<String> = context.run_cmd(&mut cmd).await?;
    match value {
      Some(v) => output.value.done(v)?,
      None => output
        .value
        .done_exception(Exception::KeyNotFound(input.key).to_string())?,
    };
    Ok(())
  }
}
