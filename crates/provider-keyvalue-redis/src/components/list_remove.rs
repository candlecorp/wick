use vino_interface_keyvalue::generated::list_remove::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::lrem(&input.key, 1, &input.value);
  let _num: u32 = context.run_cmd(&mut cmd).await?;

  output.value.done(Payload::success(&input.key))?;

  Ok(())
}
