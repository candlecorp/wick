use wafl_interface_keyvalue::set_scan::*;

use crate::components::generated::set_scan::*;
use crate::Error;

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
      .arg(&input.count);

    let (cursor, values): (String, Vec<String>) = context.run_cmd(&mut cmd).await?;
    output.values.done(values)?;
    output.cursor.done(cursor)?;

    Ok(state)
  }
}
