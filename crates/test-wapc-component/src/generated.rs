use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack::{
  deserialize,
  serialize,
};
use vino_component::v0::Payload;
use vino_component::Packet;

#[cfg(feature = "guest")]
extern crate wapc_guest as guest;
#[cfg(feature = "guest")]
use guest::prelude::*;

#[cfg(feature = "guest")]
pub struct Handlers {}

#[cfg(feature = "guest")]
impl Handlers {
  pub fn register_job(f: fn(Inputs, Outputs) -> HandlerResult<()>) {
    *JOB.write().unwrap() = Some(f);
    register_function(&"job", job_wrapper);
  }
}

#[cfg(feature = "guest")]
lazy_static::lazy_static! {
static ref JOB: std::sync::RwLock<Option<fn(Inputs, Outputs) -> HandlerResult<()>>> = std::sync::RwLock::new(None);
}

#[cfg(feature = "guest")]
fn job_wrapper(input_payload: &[u8]) -> CallResult {
  let (inv_id, input_encoded): (String, InputEncoded) = deserialize(input_payload)?;
  let outputs = get_outputs(inv_id);
  let inputs: Inputs = deserialize_inputs(input_encoded)?;
  let lock = JOB.read().unwrap().unwrap();
  let result = lock(inputs, outputs)?;
  Ok(serialize(result)?)
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct InputEncoded {
  #[serde(rename = "input")]
  pub input: Vec<u8>,
}
fn deserialize_inputs(
  args: InputEncoded,
) -> std::result::Result<
  Inputs,
  std::boxed::Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>,
> {
  Ok(Inputs {
    input: deserialize(&args.input)?,
  })
}
#[cfg(feature = "guest")]
#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct Inputs {
  #[serde(rename = "input")]
  pub input: String,
}

fn get_outputs(inv_id: String) -> Outputs {
  Outputs {
    output: GuestPortOutput {
      inv_id: inv_id.clone(),
    },
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GuestPortOutput {
  inv_id: String,
}

impl GuestPortOutput {
  pub fn send(&self, payload: String) -> CallResult {
    host_call(
      &self.inv_id,
      "output",
      "port",
      &serialize(Packet::V0(Payload::to_messagepack(payload)))?,
    )
  }
  pub fn exception(&self, message: String) -> CallResult {
    host_call(
      &self.inv_id,
      "output",
      "port",
      &serialize(Packet::V0(Payload::Exception(message)))?,
    )
  }
}

#[cfg(feature = "guest")]
#[derive(Debug, PartialEq, Clone)]
pub struct Outputs {
  pub output: GuestPortOutput,
}
