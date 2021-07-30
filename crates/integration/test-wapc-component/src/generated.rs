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
      let errmsg = format!("Guest call failed: {}", e);
      unsafe {
        wapc::__guest_error(errmsg.as_ptr(), errmsg.len() as _);
      }
      0
    }
  }
}

pub struct Dispatcher {}
impl Dispatch for Dispatcher {
  fn dispatch(op: &str, payload: &[u8]) -> CallResult {
    let payload = IncomingPayload::from_buffer(payload)?;
    let result = match op {
      "error" => error::Component::new().execute(&payload),
      "validate" => validate::Component::new().execute(&payload),
      _ => Err(Error::JobNotFound(op.to_owned())),
    }?;
    Ok(serialize(&result)?)
  }
}

pub(crate) mod error {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::{
    console_log,
    GuestPort,
    JobResult,
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
      let outputs = get_outputs(&payload.inv_id);
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
    })
  }

  #[cfg(feature = "guest")]
  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  fn get_outputs(inv_id: &str) -> Outputs {
    Outputs {
      output: GuestPortOutput { inv_id },
    }
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct GuestPortOutput<'a> {
    inv_id: &'a str,
  }

  impl<'a> GuestPort for GuestPortOutput<'a> {
    type Output = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
    fn get_invocation_id(&self) -> String {
      self.inv_id.to_owned()
    }
  }

  #[cfg(feature = "guest")]
  #[derive(Debug)]
  pub struct Outputs<'a> {
    pub output: GuestPortOutput<'a>,
  }
}
pub(crate) mod validate {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::wasm::{
    console_log,
    GuestPort,
    JobResult,
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
      let outputs = get_outputs(&payload.inv_id);
      implementation::job(inputs, outputs)
    }
  }

  fn populate_inputs(payload: &IncomingPayload) -> Result<Inputs> {
    Ok(Inputs {
      input: deserialize(payload.get("input")?)?,
    })
  }

  #[cfg(feature = "guest")]
  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  fn get_outputs(inv_id: &str) -> Outputs {
    Outputs {
      output: GuestPortOutput { inv_id },
    }
  }

  #[derive(Debug, PartialEq, Clone)]
  pub struct GuestPortOutput<'a> {
    inv_id: &'a str,
  }

  impl<'a> GuestPort for GuestPortOutput<'a> {
    type Output = String;
    fn get_name(&self) -> String {
      "output".to_string()
    }
    fn get_invocation_id(&self) -> String {
      self.inv_id.to_owned()
    }
  }

  #[cfg(feature = "guest")]
  #[derive(Debug)]
  pub struct Outputs<'a> {
    pub output: GuestPortOutput<'a>,
  }
}
