use vino_interface_keyvalue::set_get::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  trace!(?input, "set-get");
  let mut cmd = redis::Cmd::smembers(&input.key);
  let values: Vec<String> = context.run_cmd(&mut cmd).await?;
  output.values.done(Payload::success(&values))?;
  Ok(())
}
