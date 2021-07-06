use vino_codec::messagepack::{
  deserialize,
  serialize,
};
use wapc_guest::{
  host_call,
  register_function,
  CallResult,
  HandlerResult,
};

pub fn register_handlers() {
  Handlers::register_validate(super::components::validate::job);
}

#[cfg(feature = "guest")]
pub struct Handlers {}

#[cfg(feature = "guest")]
impl Handlers {
  pub fn register_validate(f: fn(validate::Inputs, validate::Outputs) -> HandlerResult<()>) {
    *VALIDATE.write().unwrap() = Some(f);
    register_function(&"validate", validate_wrapper);
  }
}

#[cfg(feature = "guest")]
lazy_static::lazy_static! {
static ref VALIDATE: std::sync::RwLock<Option<fn(validate::Inputs, validate::Outputs) -> HandlerResult<()>>> = std::sync::RwLock::new(None);
}

#[cfg(feature = "guest")]
fn validate_wrapper(input_payload: &[u8]) -> CallResult {
  use validate::*;
  let (inv_id, input_encoded): (String, InputEncoded) = deserialize(input_payload)?;
  let outputs = get_outputs(inv_id);
  let inputs: Inputs = deserialize_inputs(input_encoded)?;
  let lock = VALIDATE.read().unwrap().unwrap();
  let result = lock(inputs, outputs)?;
  Ok(serialize(result)?)
}

pub(crate) mod validate {
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_component::v0::Payload;
  use vino_component::Packet;

  use super::*;

  // Implementation for validate
  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct InputEncoded {
    #[serde(rename = "input")]
    pub input: Vec<u8>,
  }

  pub(crate) fn deserialize_inputs(
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

  pub(crate) fn get_outputs(inv_id: String) -> Outputs {
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
}
