pub use crate::components::generated::main::*;
use crate::components::*;

pub(crate) fn job(mut input: Inputs, output: OutputPorts) -> JobResult {
  let first_arg = input.argv.pop();
  if let Some(filename) = first_arg {
    println!("filename is {}", filename);
    let contents = std::fs::read_to_string(&filename)
      .map_err(|e| ComponentError::new(format!("Could not read file {}: {}", filename, e)))?;
    println!("filename contents is {}", contents);

    let mut payload = TransportMap::default();
    payload.insert("input", MessageTransport::success(&contents));

    let result = input
      .network
      .call("inner-schematic", payload)?
      .drain_port("output")?
      .pop()
      .unwrap();

    let result: u32 = result.deserialize()?;

    println!("number of bytes: {}", result);
    output.code.done(&0)?;
  } else {
    output
      .code
      .done_exception("No argument passed as first argument".to_owned())?;
  }

  Ok(())
}
