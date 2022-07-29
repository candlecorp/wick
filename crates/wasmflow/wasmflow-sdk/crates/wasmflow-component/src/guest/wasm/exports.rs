use super::imports::*;

#[allow(unsafe_code, unreachable_pub, clippy::future_not_send)]
#[no_mangle]
pub extern "C" fn __host_call_response_ready(id: i32, code: i32) {
  super::runtime::ASYNC_HOST_CALLS.with(|cell| {
    let tx = unsafe { (&*cell).get().as_mut().unwrap().remove(&id).unwrap() };
    let _ = tx.send(code);
  });
  super::runtime::exhaust_tasks();
}

#[allow(unsafe_code, unreachable_pub)]
#[no_mangle]
pub(crate) extern "C" fn __guest_call(_id: i32, _op_len: i32, _req_len: i32) -> i32 {
  panic!("sync guest calls are not supported by this crate at this time");
}

#[allow(unsafe_code, unreachable_pub)]
#[no_mangle]
pub(crate) extern "C" fn __async_guest_call(id: i32, op_len: i32, req_len: i32) {
  use std::slice;

  let buf: Vec<u8> = Vec::with_capacity(req_len as _);
  let req_ptr = buf.as_ptr();

  let opbuf: Vec<u8> = Vec::with_capacity(op_len as _);
  let op_ptr = opbuf.as_ptr();

  let (slice, op) = unsafe {
    __guest_request(id, op_ptr, req_ptr);
    (
      slice::from_raw_parts(req_ptr, req_len as _),
      slice::from_raw_parts(op_ptr, op_len as _),
    )
  };

  let op_str = ::std::str::from_utf8(op).unwrap();

  let dispatcher = super::runtime::get_dispatcher();

  if let Err(e) = &dispatcher {
    let errmsg = e.to_string();
    unsafe {
      __guest_error(id, errmsg.as_ptr(), errmsg.len());
    }
  }
  let dispatcher = dispatcher.unwrap();

  super::executor::spawn(async move {
    let result = dispatcher.dispatch(op_str, slice).await;
    let code = match result {
      Ok(result) => {
        #[allow(unsafe_code)]
        unsafe {
          __guest_response(id, result.as_ptr(), result.len());
        }
        1
      }
      Err(e) => {
        let errmsg = e.to_string();
        #[allow(unsafe_code)]
        unsafe {
          __guest_error(id, errmsg.as_ptr(), errmsg.len());
        }
        0
      }
    };
    #[allow(unsafe_code)]
    unsafe {
      __guest_call_response_ready(id, code);
    }
  });
  super::runtime::exhaust_tasks();
}
