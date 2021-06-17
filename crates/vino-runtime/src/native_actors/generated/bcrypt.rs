use crate::components::native_component_actor::NativeCallback;
use crate::{deserialize, serialize, Result};
use serde::{Deserialize, Serialize};
use vino_guest::OutputPayload;

pub(crate) struct Inputs {
  pub(crate) input: String,

  pub(crate) cost: u32,
}

pub(crate) fn inputs_list() -> Vec<String> {
  vec!["input".to_string(), "cost".to_string()]
}

pub(crate) struct Outputs<'a> {
  pub(crate) output: GuestPortOutput<'a>,
}

pub(crate) fn outputs_list() -> Vec<String> {
  vec!["output".to_string()]
}

pub(crate) struct GuestPortOutput<'a> {
  inv_id: String,
  callback: &'a NativeCallback,
}
impl<'a> GuestPortOutput<'a> {
  #[allow(dead_code)]
  pub(crate) fn send(&self, payload: String) -> Result<()> {
    (self.callback)(
      0,
      &self.inv_id,
      "",
      "output",
      &OutputPayload::MessagePack(serialize(payload)?),
    )?;
    Ok(())
  }
  #[allow(dead_code)]
  pub(crate) fn exception(&self, message: String) -> Result<()> {
    (self.callback)(
      0,
      &self.inv_id,
      "",
      "output",
      &OutputPayload::Exception(message),
    )?;
    Ok(())
  }
}

pub(crate) fn get_outputs(callback: &NativeCallback, inv_id: String) -> Outputs {
  Outputs {
    output: GuestPortOutput { inv_id, callback },
  }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub(crate) struct InputEncoded {
  #[serde(rename = "input")]
  pub(crate) input: Vec<u8>,

  #[serde(rename = "cost")]
  pub(crate) cost: Vec<u8>,
}
pub(crate) fn deserialize_inputs(
  args: InputEncoded,
) -> std::result::Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
  Ok(Inputs {
    input: deserialize(&args.input)?,

    cost: deserialize(&args.cost)?,
  })
}
