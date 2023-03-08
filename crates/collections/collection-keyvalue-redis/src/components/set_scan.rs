use wick_interface_types_keyvalue::set_scan::*;

use crate::components::generated::set_scan::*;
use crate::Error;

#[async_trait::async_trait]
impl wasmflow_sdk::v1::stateful::BatchedComponent for Component {
  type Context = crate::Context;

  async fn job(
    input: Self::Inputs,
    output: Self::Outputs,
    context: Self::Context,

    _config: Option<Self::Config>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    trace!(?input, "set-scan");
    let cursor_str = input.cursor;
    let cursor: u64 = cursor_str.parse().map_err(|_| Error::CursorConversion(cursor_str))?;
    let mut cmd = redis::cmd("sscan");
    cmd
      .arg(&input.key)
      .cursor_arg(cursor)
      .arg("MATCH")
      .arg("*")
      .arg("COUNT")
      .arg(input.count);

    let (cursor, values): (String, Vec<String>) = context.run_cmd(&mut cmd).await?;
    output.values.done(values)?;
    output.cursor.done(cursor)?;

    Ok(())
  }
}
