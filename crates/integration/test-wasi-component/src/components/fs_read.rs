pub use crate::components::generated::fs_read::*;

pub(crate) fn job(input: Inputs, output: OutputPorts) -> JobResult {
  let contents = std::fs::read_to_string(&input.filename)
    .map_err(|e| ComponentError::new(format!("Could not read file {}: {}", input.filename, e)))?;
  output.contents.done(&contents)?;
  Ok(())
}
