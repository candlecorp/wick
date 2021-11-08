use vino_interface_keyvalue::set_scan::*;

use crate::Error;

pub(crate) async fn job(input: Inputs, output: OutputPorts, context: crate::Context) -> JobResult {
  let cursor_str = input.cursor;
  let cursor: u64 = cursor_str
    .parse()
    .map_err(|_| Error::CursorConversion(cursor_str))?;
  let mut cmd = redis::cmd("sscan");
  cmd
    .arg(&input.key)
    .cursor_arg(cursor)
    .arg("MATCH")
    .arg("*")
    .arg("COUNT")
    .arg(&input.count);

  let (cursor, values): (String, Vec<String>) = context.run_cmd(&mut cmd).await?;
  output.values.done(Payload::success(&values))?;
  output.cursor.done(Payload::success(&cursor))?;

  Ok(())
}
