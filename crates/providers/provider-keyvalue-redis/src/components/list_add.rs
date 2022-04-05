use vino_interface_keyvalue::list_add::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  trace!(?input, "list-add");
  let mut cmd = redis::Cmd::rpush(&input.key, &input.values);
  let value: u32 = context.run_cmd(&mut cmd).await?;
  output.length.done(Payload::success(&value))?;

  Ok(())
}
