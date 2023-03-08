use std::convert::TryInto;

use wick_interface_types_keyvalue::list_range::*;

use crate::components::generated::list_range::*;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    trace!(?input, "list-range");
    let mut cmd = redis::Cmd::lrange(
      &input.key,
      input.start.try_into().unwrap(),
      input.end.try_into().unwrap(),
    );
    let docs: Vec<String> = context.run_cmd(&mut cmd).await?;
    output.values.done(docs)?;

    Ok(())
  }
}
