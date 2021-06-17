use crate::components::native_component_actor::NativeCallback;
use crate::{deserialize, serialize, Result};
use serde::{Deserialize, Serialize};
use vino_guest::OutputPayload;

pub(crate) struct Inputs {
  pub(crate) left: u64,

  pub(crate) right: u64,
}

pub(crate) fn inputs_list() -> Vec<String> {
  vec!["left".to_string(), "right".to_string()]
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
  pub(crate) fn send(&self, payload: u64) -> Result<()> {
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
  #[serde(rename = "left")]
  pub(crate) left: Vec<u8>,

  #[serde(rename = "right")]
  pub(crate) right: Vec<u8>,
}
pub(crate) fn deserialize_inputs(
  args: InputEncoded,
) -> std::result::Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
  Ok(Inputs {
    left: deserialize(&args.left)?,

    right: deserialize(&args.right)?,
  })
}
