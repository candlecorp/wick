use vino_interface_keyvalue::generated::set_add::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::sadd(&input.key, &input.value);
  let _num: u32 = context.run_cmd(&mut cmd).await?;
  output.key.done(&input.key)?;
  Ok(())
}
