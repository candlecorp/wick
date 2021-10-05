use crate::generated::reverse_uppercase::*;
use crate::generated::{
  reverse,
  uppercase,
};

pub(crate) fn job(input: Inputs, output: OutputPorts) -> JobResult {
  let reverse_inputs = reverse::Inputs { input: input.input };
  let mut result: reverse::Outputs = input.link.call("reverse", reverse_inputs)?.into();

  let payload: String = result.output()?.try_next_into()?;

  let uppercase_inputs = uppercase::Inputs { input: payload };

  let mut result: uppercase::Outputs = input.link.call("uppercase", uppercase_inputs)?.into();

  let payload: String = result.output()?.try_next_into()?;

  output.output.done(&payload)?;

  Ok(())
}
