use vino_interface_keyvalue::generated::key_set::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::set(&input.key, &input.value);
  let _value: String = context.run_cmd(&mut cmd).await?;
  output.key.done(&input.key)?;
  Ok(())
}