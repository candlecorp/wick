use vino_interface_keyvalue::generated::set_remove::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::srem(&input.key, &input.value);
  let _num: u32 = context.run_cmd(&mut cmd).await?;
  output.value.done(Payload::success(&input.value))?;
  Ok(())
}
