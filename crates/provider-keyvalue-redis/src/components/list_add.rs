use vino_interface_keyvalue::generated::list_add::*;

use crate::error::Exception;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::rpush(&input.key, &input.value);
  let value: u32 = context.run_cmd(&mut cmd).await?;
  if value == 0 {
    output
      .key
      .done_exception(Exception::NothingToDelete(input.key, input.value).into())?;
  } else {
    output.key.done(&input.key)?;
  }

  Ok(())
}
