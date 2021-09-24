/**********************************************
***** This file is generated, do not edit *****
***********************************************/

use vino_provider::wasm::prelude::*;

type Result<T> = std::result::Result<T, Error>;

#[no_mangle]
pub(crate) extern "C" fn __guest_call(op_len: i32, req_len: i32) -> i32 {
  use std::slice;

  let buf: Vec<u8> = Vec::with_capacity(req_len as _);
  let req_ptr = buf.as_ptr();

  let opbuf: Vec<u8> = Vec::with_capacity(op_len as _);
  let op_ptr = opbuf.as_ptr();

  let (slice, op) = unsafe {
    wapc::__guest_request(op_ptr, req_ptr);
    (
      slice::from_raw_parts(req_ptr, req_len as _),
      slice::from_raw_parts(op_ptr, op_len as _),
    )
  };

  let op_str = ::std::str::from_utf8(op).unwrap();

  match Dispatcher::dispatch(op_str, slice) {
    Ok(response) => {
      unsafe { wapc::__guest_response(response.as_ptr(), response.len()) }
      1
    }
    Err(e) => {
      let errmsg = e.to_string();
      unsafe {
        wapc::__guest_error(errmsg.as_ptr(), errmsg.len() as _);
      }
      0
    }
  }
}

static ALL_COMPONENTS: &[&str] = &["copy", "error", "reverse", "uppercase", "validate"];

pub struct Dispatcher {}
impl Dispatch for Dispatcher {
  fn dispatch(op: &str, payload: &[u8]) -> CallResult {
    let payload = IncomingPayload::from_buffer(payload)?;
    let result = match op {
      "copy" => copy::Component::new().execute(&payload),
      "error" => error::Component::new().execute(&payload),
      "reverse" => reverse::Component::new().execute(&payload),
      "uppercase" => uppercase::Component::new().execute(&payload),
      "validate" => validate::Component::new().execute(&payload),
      _ => Err(Error::ComponentNotFound(
        op.to_owned(),
        ALL_COMPONENTS.join(", "),
      )),
    }?;
    Ok(serialize(&result)?)
  }
}

pub(crate) mod copy {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::error::ComponentError;
  pub use vino_provider::wasm::{
    console_log,
    JobResult,
    PortSender,
  };

  use super::*;
  use crate::components::copy as implementation;

  pub(crate) struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let inputs = populate_inputs(payload)?;
      let outputs = get_outputs();
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
      times: deserialize(payload.get("times")?)?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
    #[serde(rename = "times")]
    pub times: i8,
  }

  fn get_outputs() -> Outputs {
    Outputs {
      output: GuestPortOutput {},
    }
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct GuestPortOutput {}

  impl PortSender for GuestPortOutput {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    pub output: GuestPortOutput,
  }
}
pub(crate) mod error {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::error::ComponentError;
  pub use vino_provider::wasm::{
    console_log,
    JobResult,
    PortSender,
  };

  use super::*;
  use crate::components::error as implementation;

  pub(crate) struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let inputs = populate_inputs(payload)?;
      let outputs = get_outputs();
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  fn get_outputs() -> Outputs {
    Outputs {
      output: GuestPortOutput {},
    }
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct GuestPortOutput {}

  impl PortSender for GuestPortOutput {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    pub output: GuestPortOutput,
  }
}
pub(crate) mod reverse {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::error::ComponentError;
  pub use vino_provider::wasm::{
    console_log,
    JobResult,
    PortSender,
  };

  use super::*;
  use crate::components::reverse as implementation;

  pub(crate) struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let inputs = populate_inputs(payload)?;
      let outputs = get_outputs();
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  fn get_outputs() -> Outputs {
    Outputs {
      output: GuestPortOutput {},
    }
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct GuestPortOutput {}

  impl PortSender for GuestPortOutput {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    pub output: GuestPortOutput,
  }
}
pub(crate) mod uppercase {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::error::ComponentError;
  pub use vino_provider::wasm::{
    console_log,
    JobResult,
    PortSender,
  };

  use super::*;
  use crate::components::uppercase as implementation;

  pub(crate) struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let inputs = populate_inputs(payload)?;
      let outputs = get_outputs();
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  fn get_outputs() -> Outputs {
    Outputs {
      output: GuestPortOutput {},
    }
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct GuestPortOutput {}

  impl PortSender for GuestPortOutput {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    pub output: GuestPortOutput,
  }
}
pub(crate) mod validate {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::error::ComponentError;
  pub use vino_provider::wasm::{
    console_log,
    JobResult,
    PortSender,
  };

  use super::*;
  use crate::components::validate as implementation;

  pub(crate) struct Component {}

  impl Component {
    pub fn new() -> Self {
      Self {}
    }
  }
  impl WapcComponent for Component {
    fn execute(&self, payload: &IncomingPayload) -> JobResult {
      let inputs = populate_inputs(payload)?;
      let outputs = get_outputs();
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  fn get_outputs() -> Outputs {
    Outputs {
      output: GuestPortOutput {},
    }
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct GuestPortOutput {}

  impl PortSender for GuestPortOutput {
    type PayloadType = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
  }

  #[derive(Debug)]
  pub struct Outputs {
    pub output: GuestPortOutput,
  }
}
