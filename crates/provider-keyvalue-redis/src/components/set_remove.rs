use vino_interface_keyvalue::set_remove::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::srem(&input.key, &input.values);
  let num: u32 = context.run_cmd(&mut cmd).await?;
  output.num.done(Payload::success(&num))?;
  Ok(())
}
