use vino_interface_keyvalue::generated::set_get::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::smembers(&input.key);
  let values: Vec<String> = context.run_cmd(&mut cmd).await?;
  output.values.done(Payload::success(&values))?;
  Ok(())
}
