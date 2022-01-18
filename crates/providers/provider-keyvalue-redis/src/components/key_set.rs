use vino_interface_keyvalue::key_set::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::set(&input.key, &input.value);
  let _value: String = context.run_cmd(&mut cmd).await?;
  output.result.done(Payload::success(&true))?;
  Ok(())
}
