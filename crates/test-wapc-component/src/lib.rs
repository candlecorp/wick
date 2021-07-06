mod generated;

pub(crate) mod components {
  pub(crate) mod validate;
}

#[no_mangle]
pub fn wapc_init() {
  generated::register_handlers();
}
