use vino_interface_keyvalue::delete::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::del(&input.keys);
  let num: u32 = context.run_cmd(&mut cmd).await?;
  output.num.done(Payload::success(&num))?;
  Ok(())
}
