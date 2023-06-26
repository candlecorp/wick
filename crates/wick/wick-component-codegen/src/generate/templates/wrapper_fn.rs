use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use wick_interface_types::OperationSignature;

use crate::generate::dependency::Dependency;
use crate::generate::expand_type::expand_type;
use crate::generate::ids::*;
use crate::generate::{config, Direction};

pub(crate) fn gen_wrapper_fn(config: &mut config::Config, component: &Ident, op: &OperationSignature) -> TokenStream {
  let impl_name = id(&snake(op.name()));
  let wrapper_name = op_wrapper_name(op);
  let wrapper_id = id(&wrapper_name);
  let input_pairs = op
    .inputs()
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_type = expand_type(config, Direction::In, false, &i.ty);
      quote! {(#port_name, #port_type)}
    })
    .collect_vec();
  let inputs = op.inputs().iter().map(|i| id(&snake(&i.name))).collect_vec();
  let outputs_name = id(&op_outputs_name(op));
  let sanitized_input_names = if inputs.is_empty() {
    quote! {config}
  } else {
    quote! {(config, #(#inputs,)*)}
  };

  let raw = if config.raw {
    quote! {raw:true}
  } else {
    quote! {raw:false}
  };
  config.add_dep(Dependency::WickPacket);
  config.add_dep(Dependency::WasmRs);
  config.add_dep(Dependency::WasmRsRuntime);
  let config_id = id(&generic_config_id());

  quote! {
    fn #wrapper_id(mut input: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>) -> std::result::Result<wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>,Box<dyn std::error::Error + Send + Sync>> {
      let (channel, rx) = wasmrs_rx::FluxChannel::<wasmrs::RawPayload, wasmrs::PayloadError>::new_parts();
      let outputs = #impl_name::#outputs_name::new(channel.clone());

      runtime::spawn(#wrapper_name,async move {
        let #sanitized_input_names = wick_component::payload_fan_out!(input, #raw, Box<dyn std::error::Error + Send + Sync>, #impl_name::#config_id, [#(#input_pairs,)*]);
         let config = match config.await {
          Ok(Ok(config)) => {
            config
          },
          _ => {
            let _ = channel.send_result(
              wick_packet::Packet::component_error("Component sent invalid context").into(),
            );
            return;
          }
        };
        if let Err(e) = #component::#impl_name(#(Box::pin(#inputs),)* outputs, config).await {
          let _ = channel.send_result(
            wick_packet::Packet::component_error(e.to_string()).into(),
          );
        }
      });

      Ok(Box::pin(rx))
    }
  }
}
