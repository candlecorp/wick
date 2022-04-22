pub use crate::components::generated::main::*;

pub(crate) fn job(mut input: Inputs, output: OutputPorts) -> JobResult {
  let first_arg = input.argv.pop();
  if let Some(filename) = first_arg {
    println!("filename is {}", filename);
    let contents = std::fs::read_to_string(&filename)
      .map_err(|e| ComponentError::new(format!("Could not read file {}: {}", filename, e)))?;
    println!("filename contents is {}", contents);
    let code = if !contents.is_empty() { 0 } else { 1 };
    output.code.done(&code)?;
  } else {
    output
      .code
      .done_exception("No argument passed as first argument".to_owned())?;
  }

  Ok(())
}
