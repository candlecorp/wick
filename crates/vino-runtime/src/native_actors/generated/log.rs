use crate::components::native_component_actor::NativeCallback;
use crate::{deserialize, serialize, Result};
use serde::{Deserialize, Serialize};
use vino_guest::OutputPayload;

pub(crate) struct Inputs {
    pub input: String,
}

pub(crate) fn inputs_list() -> Vec<String> {
    vec!["input".to_string()]
}

pub struct Outputs<'a> {
    pub output: GuestPortOutput<'a>,
}

pub(crate) fn outputs_list() -> Vec<String> {
    vec!["output".to_string()]
}

pub struct GuestPortOutput<'a> {
    inv_id: String,
    callback: &'a NativeCallback,
}
impl<'a> GuestPortOutput<'a> {
    #[allow(dead_code)]
    pub fn send(&self, payload: String) -> Result<()> {
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
    pub fn exception(&self, message: String) -> Result<()> {
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

pub fn get_outputs(callback: &NativeCallback, inv_id: String) -> Outputs {
    Outputs {
        output: GuestPortOutput { inv_id, callback },
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct InputEncoded {
    #[serde(rename = "input")]
    pub input: Vec<u8>,
}
pub(crate) fn deserialize_inputs(
    args: InputEncoded,
) -> std::result::Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
        input: deserialize(&args.input)?,
    })
}