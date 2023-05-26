use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_config::config::OperationSignature;

use crate::generate::dependency::Dependency;
use crate::generate::expand_type::expand_type;
use crate::generate::ids::*;
use crate::generate::{config, f, Direction};

pub(crate) fn op_outputs(config: &mut config::Config, op: &OperationSignature) -> TokenStream {
  let outputs_name = id(&op_outputs_name(op));
  let broadcast_statements = op.outputs().iter().map(|i| {
    let field_name = id(&snake(&i.name));
    quote! {
      self.#field_name.error(&err);
    }
  });
  let output_port_fields = op
    .outputs()
    .iter()
    .map(|i| {
      let port_field_name = id(&snake(&i.name));
      let port_type = expand_type(config, Direction::Out, false, &i.ty);
      quote! {pub(crate) #port_field_name: wick_packet::Output<#port_type>}
    })
    .collect_vec();
  let output_ports_new = op
    .outputs()
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_field_name = id(&snake(&i.name));
      quote! {#port_field_name: wick_packet::Output::new(#port_name, channel.clone())}
    })
    .collect_vec();

  let outputs = f::gen_if(
    config.output_structs,
    || {
      config.add_dep(Dependency::WickPacket);
      config.add_dep(Dependency::WasmRsRx);
      config.add_dep(Dependency::WasmRs);
    },
    quote! {

    pub struct #outputs_name {
      #[allow(unused)]
      #(#output_port_fields,)*
    }

    impl #outputs_name {
      pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
        Self {
          #(#output_ports_new,)*
        }
      }

      #[allow(unused)]
      pub fn broadcast_err(&mut self, err: impl AsRef<str>) {
        #(#broadcast_statements)*
      }
    }},
  );

  quote! {
    #outputs
  }
}
