#[link(wasm_import_module = "wapc")]
extern "C" {
  #[doc(hidden)]
  pub fn __console_log(ptr: *const u8, len: usize);
  #[doc(hidden)]
  pub fn __host_call(
    bd_ptr: *const u8,
    bd_len: usize,
    ns_ptr: *const u8,
    ns_len: usize,
    op_ptr: *const u8,
    op_len: usize,
    ptr: *const u8,
    len: usize,
  ) -> i32;
  #[doc(hidden)]
  pub(crate) fn __async_host_call(
    bd_ptr: *const u8,
    bd_len: usize,
    ns_ptr: *const u8,
    ns_len: usize,
    op_ptr: *const u8,
    op_len: usize,
    ptr: *const u8,
    len: usize,
  ) -> i32;
  #[doc(hidden)]
  pub fn __host_response(id: i32, ptr: *const u8);
  #[doc(hidden)]
  pub fn __host_response_len(id: i32) -> usize;
  #[doc(hidden)]
  pub fn __host_error_len(id: i32) -> usize;
  #[doc(hidden)]
  pub fn __host_error(id: i32, ptr: *const u8);
  #[doc(hidden)]
  pub fn __guest_response(id: i32, ptr: *const u8, len: usize);
  #[doc(hidden)]
  pub fn __guest_error(id: i32, ptr: *const u8, len: usize);
  #[doc(hidden)]
  pub fn __guest_request(id: i32, op_ptr: *const u8, ptr: *const u8);
  #[doc(hidden)]
  pub fn __guest_call_response_ready(id: i32, code: i32);
}
