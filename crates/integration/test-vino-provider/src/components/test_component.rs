pub(crate) use crate::generated::test_component::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: crate::Context,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
  output.output.done(&format!("TEST: {}", input.input))?;
  Ok(())
}
