use vino_interface_keyvalue::generated::list_add::*;

use crate::error::Exception;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::rpush(&input.key, &input.value);
  let value: u32 = context.run_cmd(&mut cmd).await?;
  if value == 0 {
    output.key.done(Payload::exception(
      Exception::NothingToDelete(input.key, input.value).to_string(),
    ))?;
  } else {
    output.key.done(Payload::success(&input.key))?;
  }

  Ok(())
}
