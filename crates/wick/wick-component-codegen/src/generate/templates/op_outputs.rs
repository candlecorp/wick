use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_interface_types::OperationSignature;

use crate::generate::dependency::Dependency;
use crate::generate::expand_type::expand_type;
use crate::generate::ids::*;
use crate::generate::{config, f, Direction};

pub(crate) fn op_outputs(config: &mut config::Config, op: &OperationSignature) -> TokenStream {
  let outputs_name = id(&op_outputs_name(op));
  let broadcast_err_statements = op.outputs().iter().map(|i| {
    let field_name = id(&snake(&i.name));
    quote! {
      self.#field_name.error(&err);
    }
  });

  let broadcast_open_statements = op.outputs().iter().map(|output| {
    let name = &output.name;
    let field_name = id(&snake(name));

    quote! {
      self.#field_name.open_bracket();
    }
  });

  let broadcast_close_statements = op.outputs().iter().map(|output| {
    let name = &output.name;
    let field_name = id(&snake(name));

    quote! {
      self.#field_name.close_bracket();
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
    .enumerate()
    .map(|(i, out)| {
      let port_name = &out.name;
      let port_field_name = id(&snake(&out.name));
      if i < op.outputs.len() - 1 {
        quote! {#port_field_name: wick_packet::Output::new(#port_name, channel.clone())}
      } else {
        quote! {#port_field_name: wick_packet::Output::new(#port_name, channel)}
      }
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
      pub fn broadcast_open(&mut self) {
        #(#broadcast_open_statements)*
      }

      #[allow(unused)]
      pub fn broadcast_close(&mut self) {
        #(#broadcast_close_statements)*
      }

      #[allow(unused)]
      pub fn broadcast_err(&mut self, err: impl AsRef<str>) {
        #(#broadcast_err_statements)*
      }
    }},
  );

  quote! {
    #outputs
  }
}
