//! Vino's WebAssembly provider implementations. This module
//!  is only active with the feature `wasm` enabled.
//!
#![allow(
  unsafe_code,
  missing_debug_implementations,
  missing_copy_implementations
)]

use std::collections::HashMap;

#[doc(hidden)]
pub mod wapc;

use vino_codec::messagepack::{
  deserialize,
  serialize,
};

/// Errors for WebAssembly providers.
pub mod error;
/// The WebAssembly implementation of a Port Sender.
pub mod port_sender;
pub use error::Error;
pub use port_sender::PortSender;
use vino_packet::{
  v0,
  Packet,
};

use crate::wasm::wapc::*;
type Result<T> = std::result::Result<T, Error>;

/// The return signature for WebAssembly jobs.
pub type JobResult = Result<()>;

#[doc(hidden)]
// This is meant for code generation consumers.
pub type CallResult = Result<Vec<u8>>;

/// Common imports for WebAssembly providers and components.
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

/// Send a [Packet] out the named port.
pub fn port_send(port_name: &str, packet: v0::Payload) -> CallResult {
  host_call(
    "",
    port_name,
    OutputSignal::Output.as_str(),
    &serialize(&Packet::V0(packet))?,
  )
}

/// Send a [Packet] out the named port and immediately close it.
pub fn port_send_close(port_name: &str, packet: v0::Payload) -> CallResult {
  host_call(
    "",
    port_name,
    OutputSignal::OutputDone.as_str(),
    &serialize(&Packet::V0(packet))?,
  )
}

/// Close the referenced port.
pub fn port_close(port_name: &str) -> CallResult {
  host_call("", port_name, OutputSignal::Done.as_str(), &[])
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
/// Request a line to be printed on the native host.
pub fn console_log(s: &str) {
  unsafe {
    __console_log(s.as_ptr(), s.len());
  }
}

/// A map of port name to payload message.
pub struct IncomingPayload {
  encoded: HashMap<String, Vec<u8>>,
}

impl IncomingPayload {
  /// Decode MessagePack bytes into an [IncomingPayload].
  pub fn from_buffer(buffer: &[u8]) -> Result<Self> {
    let input_encoded: HashMap<String, Vec<u8>> = deserialize(buffer)?;

    Ok(Self {
      encoded: input_encoded,
    })
  }
  /// Get the contained bytes for the specified port.
  pub fn get(&self, port: &str) -> Result<&Vec<u8>> {
    self
      .encoded
      .get(port)
      .ok_or_else(|| Error::MissingInput(port.to_owned()))
  }
}

/// The trait for WaPC-based WebAssembly components.
pub trait WapcComponent {
  /// This method takes an incoming payload and is expected to return nothing.
  /// Vino expects execution output to be sent over the WaPC protocol via host calls.
  fn execute(&self, payload: &IncomingPayload) -> Result<()>;
}

#[doc(hidden)]
pub trait Dispatch {
  fn dispatch(op: &str, payload: &[u8]) -> CallResult;
}
