use proc_macro2::{Ident, TokenStream};
use quote::quote;
use wick_config::config::{Binding, InterfaceDefinition};
use wick_interface_types::OperationSignature;

use crate::generate::dependency::Dependency;
use crate::generate::ids::*;
use crate::generate::{config, f};

pub(crate) fn gen_component_impls<'a>(
  gen_config: &mut config::Config,
  component_name: &Ident,
  ops: impl Iterator<Item = &'a OperationSignature>,
  required: Vec<Binding<InterfaceDefinition>>,
) -> TokenStream {
  let provided = f::gen_if(
    !required.is_empty(),
    || {},
    super::provided_struct(gen_config, &required),
  );
  let imported_components = super::imported_components(gen_config, required);
  // let imported_components = super::imported_components(gen_config, required);
  let register_operations = register_operations(gen_config, component_name, ops);
  gen_config.add_dep(Dependency::WickPacket);
  gen_config.add_dep(Dependency::WasmRs);
  gen_config.add_dep(Dependency::WasmRsCodec);
  quote! {
    #[no_mangle]
    #[cfg(target_family = "wasm")]
    extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
      wick_component::wasmrs_guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
      wick_component::wasmrs_guest::register_request_response("wick", "__setup", Box::new(__setup));
      #(#register_operations)*
    }

    #imported_components
    #provided

  }
}

fn register_operations<'a>(
  _config: &config::Config,
  component: &Ident,
  op: impl Iterator<Item = &'a OperationSignature>,
) -> Vec<TokenStream> {
  op.map(|op| {
    let name = id(&op_wrapper_name(op));
    let string = op.name();

    quote! {
      wick_component::wasmrs_guest::register_request_channel("wick", #string, Box::new(#component::#name));
    }
  })
  .collect()
}
