mod components;
mod generated;

#[no_mangle]
pub fn wapc_init() {
  generated::register_handlers();
}
