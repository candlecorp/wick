use vino_interface_keyvalue::generated::exists::*;

use crate::error::Exception;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::exists(&input.key);
  let value: u32 = context.run_cmd(&mut cmd).await?;
  if value == 0 {
    output
      .key
      .done_exception(Exception::KeyNotFound(input.key).into())?;
  } else {
    output.key.done(&input.key)?;
  };
  Ok(())
}
