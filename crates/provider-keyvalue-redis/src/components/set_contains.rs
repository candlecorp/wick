use vino_interface_keyvalue::set_contains::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::sismember(&input.key, &input.member);
  let exists: bool = context.run_cmd(&mut cmd).await?;
  output.exists.done(Payload::success(&exists))?;
  Ok(())
}
