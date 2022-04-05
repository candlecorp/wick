use vino_interface_keyvalue::set_add::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  trace!(?input, "set-add");
  let mut cmd = redis::Cmd::sadd(&input.key, &input.values);
  let num: u32 = context.run_cmd(&mut cmd).await?;
  output.length.done(Payload::success(&num))?;
  Ok(())
}
