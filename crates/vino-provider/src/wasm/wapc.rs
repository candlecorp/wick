#[link(wasm_import_module = "wapc")]
extern "C" {
  pub fn __console_log(ptr: *const u8, len: usize);
  pub fn __host_call(
    bd_ptr: *const u8,
    bd_len: usize,
    ns_ptr: *const u8,
    ns_len: usize,
    op_ptr: *const u8,
    op_len: usize,
    ptr: *const u8,
    len: usize,
  ) -> usize;
  pub fn __host_response(ptr: *const u8);
  pub fn __host_response_len() -> usize;
  pub fn __host_error_len() -> usize;
  pub fn __host_error(ptr: *const u8);
  pub fn __guest_response(ptr: *const u8, len: usize);
  pub fn __guest_error(ptr: *const u8, len: usize);
  pub fn __guest_request(op_ptr: *const u8, ptr: *const u8);
}
