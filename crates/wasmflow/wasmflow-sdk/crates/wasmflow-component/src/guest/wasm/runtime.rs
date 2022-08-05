use std::cell::UnsafeCell;
use std::collections::HashMap;

use tokio::sync::oneshot;
use wasmflow_codec::messagepack::serialize;

use super::error::Error;
use super::imports::*;
use crate::guest::ephemeral::wasm::Dispatcher;
use crate::guest::wasm::BoxedFuture;
use crate::guest::BoxedError;
use crate::OutputSignal;
thread_local! {
  pub(super) static ASYNC_HOST_CALLS: UnsafeCell<HashMap<i32,oneshot::Sender<i32>>>  = UnsafeCell::new(HashMap::new());
}

thread_local! {
  pub(super) static DISPATCHER: UnsafeCell<Option<Box<dyn Dispatcher + Sync + Send>>>  = UnsafeCell::new(None);
}
type CallResult = Result<Vec<u8>, BoxedError>;

pub fn exhaust_tasks() {
  yielding_executor::single_threaded::run_while(move || {
    let num_in_flight = ASYNC_HOST_CALLS.with(|cell| {
      #[allow(unsafe_code)]
      unsafe {
        let map: &HashMap<i32, oneshot::Sender<i32>> = &*cell.get();
        map.len()
      }
    });
    num_in_flight == 0
  });
}

pub fn register_dispatcher(dispatcher: Box<dyn Dispatcher + Send + Sync>) {
  #[allow(unsafe_code)]
  DISPATCHER.with(|cell| {
    let option: &mut Option<Box<dyn Dispatcher + Sync + Send>> = unsafe { &mut *cell.get() };
    option.replace(dispatcher);
  });
}

pub fn get_dispatcher() -> Result<&'static (dyn Dispatcher + Sync + Send), Error> {
  #[allow(unsafe_code)]
  DISPATCHER.with(|cell| {
    let option: &mut Option<Box<dyn Dispatcher + Sync + Send>> = unsafe { &mut *cell.get() };
    option.as_deref().ok_or(Error::Async)
  })
}

/// The function through which all host calls take place.
pub fn host_call(binding: &str, ns: &str, op: &str, msg: &[u8]) -> CallResult {
  #[allow(unsafe_code)]
  let id = unsafe {
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

  if id == 0 {
    // call was not successful
    #[allow(unsafe_code)]
    let len = unsafe { __host_error_len(id) };
    let buf = Vec::with_capacity(len);
    let retptr = buf.as_ptr();
    #[allow(unsafe_code)]
    let _slice = unsafe {
      __host_error(id, retptr);
      std::slice::from_raw_parts(retptr, len)
    };
    Err(Box::new(super::Error::HostError(
      String::from_utf8_lossy(_slice).to_string(),
    )))
  } else {
    // call succeeded
    #[allow(unsafe_code)]
    let len = unsafe { __host_response_len(id) };
    let buf = Vec::with_capacity(len);
    let retptr = buf.as_ptr();
    #[allow(unsafe_code)]
    let slice = unsafe {
      __host_response(id, retptr);
      std::slice::from_raw_parts(retptr, len)
    };
    Ok(slice.to_vec())
  }
}

#[cold]
#[inline(never)]
/// Request a line to be printed on the native host.
pub fn console_log(s: &str) {
  #[allow(unsafe_code)]
  unsafe {
    __console_log(s.as_ptr(), s.len());
  }
}

#[allow(clippy::future_not_send)]
#[must_use]
/// The function through which all host calls take place.
pub fn async_host_call<'a>(binding: &'a str, ns: &'a str, op: &'a str, msg: &'a [u8]) -> BoxedFuture<CallResult> {
  let (send, recv) = tokio::sync::oneshot::channel();

  #[allow(unsafe_code)]
  let id = unsafe {
    __async_host_call(
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

  ASYNC_HOST_CALLS.with(|cell| {
    #[allow(unsafe_code)]
    let map = unsafe { (&*cell).get().as_mut().unwrap() };
    map.insert(id, send);
  });

  Box::pin(async move {
    if id == 0 {
      // call was not successful
      #[allow(unsafe_code)]
      let errlen = unsafe { __host_error_len(id) };

      let mut buf = Vec::with_capacity(errlen);
      let retptr = buf.as_mut_ptr();

      #[allow(unsafe_code)]
      unsafe {
        __host_error(id, retptr);
        buf.set_len(errlen);
      }
      Ok(buf)
    } else {
      // call succeeded
      match recv.await {
        Ok(_code) => {
          #[allow(unsafe_code)]
          let len = unsafe { __host_response_len(id) };

          let mut buf = Vec::with_capacity(len);
          let retptr = buf.as_mut_ptr();

          #[allow(unsafe_code)]
          unsafe {
            __host_response(id, retptr);
            buf.set_len(len);
          }
          Ok(buf)
        }
        Err(_) => Err(Error::Async.into()),
      }
    }
  })
}

fn serialize_payload(id: u32, packet: Option<wasmflow_packet::Packet>) -> Result<Vec<u8>, Error> {
  let bytes = match packet {
    Some(packet) => {
      let bytes = serialize(&packet)?;
      let mut payload = Vec::with_capacity(bytes.len() + 4);
      payload.extend_from_slice(&id.to_be_bytes());
      payload.extend(bytes.into_iter());
      payload
    }
    None => {
      let mut payload = Vec::with_capacity(4);
      payload.extend_from_slice(&id.to_be_bytes());
      payload
    }
  };
  Ok(bytes)
}

/// Send a [Packet] out the named port.
pub fn port_send(port_name: &str, id: u32, packet: wasmflow_packet::Packet) -> Result<(), Error> {
  let bytes = serialize_payload(id, Some(packet))?;
  host_call("0", port_name, OutputSignal::Output.as_str(), &bytes).map_err(Error::Protocol)?;
  Ok(())
}

/// Send a [Packet] out the named port and immediately close it.
pub fn port_send_close(port_name: &str, id: u32, packet: wasmflow_packet::Packet) -> Result<(), Error> {
  let bytes = serialize_payload(id, Some(packet))?;
  host_call("0", port_name, OutputSignal::OutputDone.as_str(), &bytes).map_err(Error::Protocol)?;
  Ok(())
}

/// Close the referenced port.
pub fn port_close(port_name: &str, id: u32) -> Result<(), Error> {
  let bytes = serialize_payload(id, None)?;
  host_call("0", port_name, OutputSignal::Done.as_str(), &bytes).map_err(Error::Protocol)?;
  Ok(())
}
