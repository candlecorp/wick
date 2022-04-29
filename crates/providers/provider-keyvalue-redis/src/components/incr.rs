pub use vino_interface_keyvalue::incr::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  trace!(?input, "incr");

  let mut cmd = redis::Cmd::incr(&input.key, input.amount);
  let value: String = context.run_cmd(&mut cmd).await?;
  let num: i64 = value
    .parse()
    .map_err(|_| format!("Could not parse string into integer. Value was '{}' ", value))?;
  output.output.done(Payload::success(&num))?;

  Ok(())
}
