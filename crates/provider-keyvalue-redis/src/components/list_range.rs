use std::convert::TryInto;

use vino_interface_keyvalue::generated::list_range::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::lrange(
    &input.key,
    input.start.try_into().unwrap(),
    input.end.try_into().unwrap(),
  );
  let docs: Vec<String> = context.run_cmd(&mut cmd).await?;
  output.values.done(Payload::success(&docs))?;

  Ok(())
}
