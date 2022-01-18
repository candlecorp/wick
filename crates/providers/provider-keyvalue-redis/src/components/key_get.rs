use vino_interface_keyvalue::key_get::*;

use crate::error::Exception;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::get(&input.key);
  let value: Option<String> = context.run_cmd(&mut cmd).await?;
  match value {
    Some(v) => output.value.done(Payload::success(&v))?,
    None => output.value.done(Payload::exception(
      Exception::KeyNotFound(input.key).to_string(),
    ))?,
  };
  Ok(())
}
