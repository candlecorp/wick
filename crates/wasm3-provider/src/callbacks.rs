use std::cell::RefCell;
use std::sync::Arc;

use wapc::ModuleState;
use wasm3::CallContext;

type ModState = Arc<RefCell<ModuleState>>;

pub(crate) fn host_call(
  ctx: &CallContext,
  bd_ptr: i32,
  bd_len: i32,
  ns_ptr: i32,
  ns_len: i32,
  op_ptr: i32,
  op_len: i32,
  ptr: i32,
  len: i32,
  host: ModState,
) -> i32 {
  let vec = get_vec_from_memory(ctx, ptr, len);
  let bd_vec = get_vec_from_memory(ctx, bd_ptr, bd_len);
  let bd = ::std::str::from_utf8(&bd_vec).unwrap();
  let ns_vec = get_vec_from_memory(ctx, ns_ptr, ns_len);
  let ns = ::std::str::from_utf8(&ns_vec).unwrap();
  let op_vec = get_vec_from_memory(ctx, op_ptr, op_len);
  let op = ::std::str::from_utf8(&op_vec).unwrap();

  let result = host.borrow().do_host_call(bd, ns, op, &vec);
  if let Ok(r) = result {
    r
  } else {
    0
  }
}

pub(crate) fn guest_request(ctx: &CallContext, op_ptr: i32, ptr: i32, host: ModState) {
  if let Some(inv) = host.borrow().get_guest_request() {
    write_bytes_to_memory(ctx, ptr, &inv.msg);
    write_bytes_to_memory(ctx, op_ptr, &inv.operation.as_bytes());
  }
}

pub(crate) fn host_response(ctx: &CallContext, ptr: i32, host: ModState) {
  if let Some(ref r) = host.borrow().get_host_response() {
    write_bytes_to_memory(ctx, ptr, &r);
  }
}

pub(crate) fn host_response_length(_ctx: &CallContext, host: ModState) -> i32 {
  host.borrow().get_host_response().unwrap_or(vec![]).len() as _
}

pub(crate) fn console_log(ctx: &CallContext, ptr: i32, len: i32, host: ModState) {
  let vec = get_vec_from_memory(ctx, ptr, len);
  let msg = std::str::from_utf8(&vec).unwrap();
  host.borrow().do_console_log(msg);
}

// Sets the guest response by telling the host "you can find the response binary here, and it's x bytes"
pub(crate) fn guest_response(ctx: &CallContext, ptr: i32, len: i32, host: ModState) {
  let vec = get_vec_from_memory(ctx, ptr, len);
  host.borrow().set_guest_response(vec);
}

// Sets the guest error by telling the host "you can find the error binary here, and it's x bytes"
pub(crate) fn guest_error(ctx: &CallContext, ptr: i32, len: i32, host: ModState) {
  let vec = get_vec_from_memory(ctx, ptr, len);
  host
    .borrow()
    .set_guest_error(String::from_utf8(vec).unwrap());
}

// Writes the host error, if any, to the linear memory at the location supplied by the guest
pub(crate) fn host_error(ctx: &CallContext, ptr: i32, host: ModState) {
  if let Some(ref e) = host.borrow().get_host_error() {
    write_bytes_to_memory(ctx, ptr, e.as_bytes());
  }
}

// Returns the length of the host error, 0 if there is none.
pub(crate) fn host_error_length(host: ModState) -> i32 {
  host
    .borrow()
    .get_host_error()
    .unwrap_or_else(|| "".to_string())
    .len() as _
}

fn get_vec_from_memory(ctx: &CallContext, ptr: i32, len: i32) -> Vec<u8> {
  let data = unsafe { &*ctx.memory() };

  data[ptr as usize..][..len as usize].to_vec()
}

fn write_bytes_to_memory(ctx: &CallContext, ptr: i32, slice: &[u8]) {
  unsafe {
    (&mut *ctx.memory_mut())[ptr as usize..][..slice.len()].copy_from_slice(slice);
  };
}
