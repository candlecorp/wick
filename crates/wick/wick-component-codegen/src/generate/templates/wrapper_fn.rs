use proc_macro2::{Ident, TokenStream};
use quote::quote;
use wick_interface_types::OperationSignature;

use crate::generate::config;
use crate::generate::dependency::Dependency;
use crate::generate::ids::*;

pub(crate) fn gen_wrapper_fn(config: &mut config::Config, component: &Ident, op: &OperationSignature) -> TokenStream {
  let impl_name = id(&snake(op.name()));
  let wrapper_name = op_wrapper_name(op);
  let wrapper_id = id(&wrapper_name);
  let outputs_name = id("Outputs");

  config.add_dep(Dependency::WickPacket);
  config.add_dep(Dependency::WasmRs);
  config.add_dep(Dependency::WasmRsRuntime);

  quote! {
    fn #wrapper_id(input: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>) -> std::result::Result<wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>,wick_component::BoxError> {
      let (channel, rx) = wasmrs_rx::FluxChannel::<wasmrs::RawPayload, wasmrs::PayloadError>::new_parts();
      let outputs = #impl_name::#outputs_name::with_channel(channel.clone());

      runtime::spawn(#wrapper_name,async move {
         let (config, inputs) = #impl_name::process_incoming(input);
         let config = match config.await {
          Ok(config) => {
            config
          },
          Err(e) => {
            let _ = channel.send_result(
              wick_packet::Packet::component_error(format!("Component sent invalid context: {}", e)).into(),
            );
            return;
          }
        };
        use self::#impl_name::Operation;
        if let Err(e) = #component::#impl_name(inputs, outputs, config).await {
          let _ = channel.send_result(
            wick_packet::Packet::component_error(e.to_string()).into(),
          );
        }
      });

      Ok(Box::pin(rx))
    }
  }
}
