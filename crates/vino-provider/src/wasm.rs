#![allow(
  unsafe_code,
  missing_docs,
  missing_debug_implementations,
  missing_copy_implementations
)]

use std::collections::HashMap;
pub mod wapc;

use vino_codec::messagepack::{
  deserialize,
  serialize,
};

pub mod error;
pub mod port_sender;
pub use error::Error;
pub use port_sender::PortSender;
use vino_component::{
  v0,
  Packet,
};

use crate::wasm::wapc::*;
type Result<T> = std::result::Result<T, Error>;
pub type JobResult = Result<()>;

pub type CallResult = Result<Vec<u8>>;

pub mod prelude {
  pub use super::{
    console_log,
    wapc,
    CallResult,
    Dispatch,
    Error,
    IncomingPayload,
    JobResult,
    PortSender,
    WapcComponent,
  };
  pub use crate::codec::messagepack::{
    deserialize,
    serialize,
  };
}

use crate::OutputSignal;

pub fn port_send(invocation_id: &str, port_name: &str, packet: v0::Payload) -> CallResult {
  host_call(
    invocation_id,
    port_name,
    OutputSignal::Output.as_str(),
    &serialize(&Packet::V0(packet))?,
  )
}

pub fn port_send_close(invocation_id: &str, port_name: &str, packet: v0::Payload) -> CallResult {
  host_call(
    invocation_id,
    port_name,
    OutputSignal::OutputDone.as_str(),
    &serialize(&Packet::V0(packet))?,
  )
}

pub fn port_close(invocation_id: &str, port_name: &str) -> CallResult {
  host_call(invocation_id, port_name, OutputSignal::Done.as_str(), &[])
}

/// The function through which all host calls take place.
pub(crate) fn host_call(binding: &str, ns: &str, op: &str, msg: &[u8]) -> CallResult {
  let callresult = unsafe {
    __host_call(
      binding.as_ptr(),
      binding.len(),
      ns.as_ptr(),
      ns.len(),
      op.as_ptr(),
      op.len(),
      msg.as_ptr(),
      msg.len(),
    )
  };
  if callresult != 1 {
    // call was not successful
    let errlen = unsafe { __host_error_len() };
    let buf = Vec::with_capacity(errlen);
    let retptr = buf.as_ptr();
    let _slice = unsafe {
      __host_error(retptr);
      std::slice::from_raw_parts(retptr, errlen)
    };
    Err(Error::HostError("Component execution failed".to_owned()))
  } else {
    // call succeeded
    let len = unsafe { __host_response_len() };
    let buf = Vec::with_capacity(len);
    let retptr = buf.as_ptr();
    let slice = unsafe {
      __host_response(retptr);
      std::slice::from_raw_parts(retptr, len)
    };
    Ok(slice.to_vec())
  }
}

#[cold]
#[inline(never)]
pub fn console_log(s: &str) {
  unsafe {
    __console_log(s.as_ptr(), s.len());
  }
}

pub struct IncomingPayload {
  pub inv_id: String,
  encoded: HashMap<String, Vec<u8>>,
}

impl IncomingPayload {
  pub fn from_buffer(buffer: &[u8]) -> Result<Self> {
    let (inv_id, input_encoded): (String, HashMap<String, Vec<u8>>) = deserialize(buffer)?;

    Ok(Self {
      inv_id,
      encoded: input_encoded,
    })
  }
  pub fn get(&self, field: &str) -> Result<&Vec<u8>> {
    self
      .encoded
      .get(field)
      .ok_or_else(|| Error::MissingInput(field.to_owned()))
  }
}

pub trait WapcComponent {
  fn execute(&self, payload: &IncomingPayload) -> Result<()>;
}

pub trait Dispatch {
  fn dispatch(op: &str, payload: &[u8]) -> CallResult;
}
