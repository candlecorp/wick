use std::convert::TryInto;

use vino_interface_keyvalue::list_remove::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let num: isize = input.num.try_into().unwrap_or(isize::MAX);
  let mut cmd = redis::Cmd::lrem(&input.key, num, &input.value);
  let num: u32 = context.run_cmd(&mut cmd).await?;

  output.num.done(Payload::success(&num))?;

  Ok(())
}
