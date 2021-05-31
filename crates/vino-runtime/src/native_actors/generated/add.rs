use crate::{connection_downstream::ConnectionDownstream, deserialize, serialize, Result};
use serde::{Deserialize, Serialize};
use vino_guest::OutputPayload;

pub(crate) struct Inputs {
    pub left: u64,

    pub right: u64,
}

pub(crate) fn inputs_list() -> Vec<String> {
    vec!["left".to_string(), "right".to_string()]
}

#[derive(Debug, PartialEq, Clone)]
pub struct Outputs {
    pub output: GuestPortOutput,
}

pub(crate) fn outputs_list() -> Vec<String> {
    vec!["output".to_string()]
}

#[derive(Debug, PartialEq, Clone)]
pub struct GuestPortOutput {
    connection: ConnectionDownstream,
}
impl GuestPortOutput {
    #[allow(dead_code)]
    pub fn send(&self, payload: u64) -> Result<()> {
        self.connection.send(
            "output".to_string(),
            serialize(OutputPayload::Bytes(serialize(payload)?))?,
        )
    }
    #[allow(dead_code)]
    pub fn exception(&self, message: String) -> Result<()> {
        self.connection.send(
            "output".to_string(),
            serialize(OutputPayload::Exception(message))?,
        )
    }
}

pub fn get_outputs(connection: ConnectionDownstream) -> Outputs {
    Outputs {
        output: GuestPortOutput { connection },
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct InputEncoded {
    #[serde(rename = "left")]
    pub left: Vec<u8>,

    #[serde(rename = "right")]
    pub right: Vec<u8>,
}
pub(crate) fn deserialize_inputs(
    args: InputEncoded,
) -> std::result::Result<
    Inputs,
    std::boxed::Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>,
> {
    Ok(Inputs {
        left: deserialize(&args.left)?,

        right: deserialize(&args.right)?,
    })
}
