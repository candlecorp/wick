use proc_macro2::{Ident, TokenStream};
use quote::quote;
use wick_config::config::{BoundInterface, OperationSignature};

use crate::generate::dependency::Dependency;
use crate::generate::ids::*;
use crate::generate::{config, f};

pub(crate) fn gen_component_impls<'a>(
  gen_config: &mut config::Config,
  component_name: &Ident,
  ops: impl Iterator<Item = &'a OperationSignature>,
  required: Vec<BoundInterface>,
) -> TokenStream {
  let provided = f::gen_if(
    !required.is_empty(),
    || {},
    super::provided_struct(gen_config, &required),
  );
  let response_streams = super::response_streams(gen_config, required);
  let register_stmts = gen_register_channels(gen_config, component_name, ops);
  gen_config.add_dep(Dependency::WickPacket);
  gen_config.add_dep(Dependency::WasmRs);
  gen_config.add_dep(Dependency::WasmRsCodec);
  quote! {
    #[no_mangle]
    #[cfg(target_family = "wasm")]
    extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
      wasmrs_guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
      wasmrs_guest::register_request_response("wick", "__setup", Box::new(__setup));
      #(#register_stmts)*
    }

    #[cfg(target_family = "wasm")]
    thread_local! {
      static __CONFIG: std::cell::UnsafeCell<Option<SetupPayload>> = std::cell::UnsafeCell::new(None);
    }

    #[cfg(target_family = "wasm")]
    #[derive(Debug, serde::Deserialize)]
    pub(crate) struct SetupPayload {
      #[allow(unused)]
      pub(crate) provided: std::collections::HashMap<String,wick_packet::ComponentReference>
    }

    #[cfg(target_family = "wasm")]
    fn __setup(input: wasmrs_rx::BoxMono<wasmrs::Payload, wasmrs::PayloadError>) -> Result<wasmrs_rx::BoxMono<wasmrs::RawPayload, wasmrs::PayloadError>, wick_component::BoxError> {
      Ok(Box::pin(async move {
        match input.await {
          Ok(payload) => {
            let input = wasmrs_codec::messagepack::deserialize::<SetupPayload>(&payload.data).unwrap();
            __CONFIG.with(|cell| {
              #[allow(unsafe_code)]
              unsafe { &mut *cell.get() }.replace(input);
            });
            Ok(wasmrs::RawPayload::new_data(None, None))
          }
          Err(e) => {
            return Err(e);
          }
        }
      }))
    }

    #[allow(unused)]
    #[cfg(target_family = "wasm")]
    pub(crate) fn get_config() -> &'static SetupPayload {
      __CONFIG.with(|cell| {
        #[allow(unsafe_code)]
        unsafe { & *cell.get() }.as_ref().unwrap()
      })
    }
    #response_streams
    #provided

  }
}

fn gen_register_channels<'a>(
  _config: &config::Config,
  component: &Ident,
  op: impl Iterator<Item = &'a OperationSignature>,
) -> Vec<TokenStream> {
  op.map(|op| {
    let name = id(&op_wrapper_name(op));
    let string = op.name();

    quote! {
      wasmrs_guest::register_request_channel("wick", #string, Box::new(#component::#name));
    }
  })
  .collect()
}
