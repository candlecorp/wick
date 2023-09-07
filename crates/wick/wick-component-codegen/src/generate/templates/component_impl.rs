use proc_macro2::{Ident, TokenStream};
use quote::quote;
use wick_config::config::{Binding, ComponentDefinition, ImportDefinition, InterfaceDefinition};
use wick_interface_types::OperationSignature;

use crate::generate::config;
use crate::generate::ids::*;

pub(crate) fn gen_component_impls<'a>(
  gen_config: &mut config::Config,
  component_name: &Ident,
  ops: impl Iterator<Item = &'a OperationSignature>,
  required: &[Binding<InterfaceDefinition>],
  imported: &[Binding<ImportDefinition>],
) -> TokenStream {
  let imported_components: Vec<_> = imported
    .iter()
    .filter_map(|i| match i.kind() {
      ImportDefinition::Component(c) => Some(Binding::<&ComponentDefinition>::new(i.id(), c)),
      ImportDefinition::Types(_) => None,
    })
    .collect();

  let provided_impl = (!required.is_empty()).then(|| super::imported_component_container("provided", required));

  let imported_impl =
    (!imported_components.is_empty()).then(|| super::imported_component_container("imported", imported));

  let required_components = super::imported_components("provided", gen_config, required);
  let imported_components = super::imported_components("imported", gen_config, &imported_components);
  let register_operations = register_operations(component_name, ops);

  quote! {
    #[no_mangle]
    #[cfg(target_family = "wasm")]
    extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
      wick_component::wasmrs_guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
      wick_component::wasmrs_guest::register_request_response("wick", "__setup", Box::new(__setup));
      #(#register_operations)*
    }

    #required_components
    #imported_components
    #provided_impl
    #imported_impl
  }
}

fn register_operations<'a>(component: &Ident, op: impl Iterator<Item = &'a OperationSignature>) -> Vec<TokenStream> {
  op.map(|op| {
    let name = id(&op_wrapper_name(op));
    let string = op.name();

    quote! {
      wick_component::wasmrs_guest::register_request_channel("wick", #string, Box::new(#component::#name));
    }
  })
  .collect()
}
