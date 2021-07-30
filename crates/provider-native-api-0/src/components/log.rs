use crate::generated::log::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> JobResult {
  println!("Logger: {}", input.input);
  output.output.done(&input.input)?;
  Ok(())
}
