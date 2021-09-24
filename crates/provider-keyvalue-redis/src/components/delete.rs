use vino_interface_keyvalue::generated::delete::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::del(&input.key);
  let _num: u32 = context.run_cmd(&mut cmd).await?;
  output.key.done(Payload::success(&input.key))?;
  Ok(())
}
